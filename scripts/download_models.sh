#!/bin/bash
#
# Model Download Script for LLM Shield
#
# Downloads pre-converted ONNX models from the model registry,
# verifies checksums, and sets up the correct directory structure.
#
# Usage:
#   ./download_models.sh [options]
#
# Options:
#   --model NAME        Download specific model (default: all)
#   --registry FILE     Path to registry.json (default: ../models/registry.json)
#   --output-dir DIR    Output directory (default: ../models/onnx)
#   --verify-only       Only verify existing models, don't download
#   --force             Force re-download even if model exists
#   --list              List available models and exit
#   --help              Show this help message
#
# Examples:
#   ./download_models.sh                           # Download all models
#   ./download_models.sh --model prompt-injection  # Download specific model
#   ./download_models.sh --verify-only             # Verify existing models
#   ./download_models.sh --list                    # List available models
#

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEFAULT_REGISTRY="${SCRIPT_DIR}/../models/registry.json"
DEFAULT_OUTPUT_DIR="${SCRIPT_DIR}/../models/onnx"

# Command-line arguments
MODEL_NAME=""
REGISTRY_FILE="${DEFAULT_REGISTRY}"
OUTPUT_DIR="${DEFAULT_OUTPUT_DIR}"
VERIFY_ONLY=false
FORCE_DOWNLOAD=false
LIST_MODELS=false

# Utility functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

show_help() {
    sed -n '2,23p' "$0" | sed 's/^# //' | sed 's/^#//'
    exit 0
}

# Parse command-line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --model)
                MODEL_NAME="$2"
                shift 2
                ;;
            --registry)
                REGISTRY_FILE="$2"
                shift 2
                ;;
            --output-dir)
                OUTPUT_DIR="$2"
                shift 2
                ;;
            --verify-only)
                VERIFY_ONLY=true
                shift
                ;;
            --force)
                FORCE_DOWNLOAD=true
                shift
                ;;
            --list)
                LIST_MODELS=true
                shift
                ;;
            --help)
                show_help
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                ;;
        esac
    done
}

# Check required tools
check_requirements() {
    local missing_tools=()

    for tool in curl jq sha256sum tar; do
        if ! command -v "$tool" &> /dev/null; then
            missing_tools+=("$tool")
        fi
    done

    if [ ${#missing_tools[@]} -gt 0 ]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        log_info "Install with: apt-get install curl jq coreutils tar"
        exit 1
    fi
}

# Load registry file
load_registry() {
    if [ ! -f "$REGISTRY_FILE" ]; then
        log_error "Registry file not found: $REGISTRY_FILE"
        exit 1
    fi

    if ! jq empty "$REGISTRY_FILE" 2>/dev/null; then
        log_error "Invalid JSON in registry file: $REGISTRY_FILE"
        exit 1
    fi

    log_success "Loaded registry: $REGISTRY_FILE"
}

# List available models
list_available_models() {
    log_info "Available models in registry:"
    echo ""

    jq -r '.models[] | "\(.name) (\(.task))\n  Version: \(.version)\n  Size: \(.size_mb)MB\n  Status: \(.status)\n"' "$REGISTRY_FILE"

    exit 0
}

# Get model info from registry
get_model_info() {
    local model_name="$1"
    jq -r --arg name "$model_name" '.models[] | select(.name == $name)' "$REGISTRY_FILE"
}

# Calculate SHA256 checksum
calculate_checksum() {
    local file="$1"
    sha256sum "$file" | awk '{print $1}'
}

# Verify file checksum
verify_checksum() {
    local file="$1"
    local expected_checksum="$2"

    log_info "Verifying checksum for $(basename "$file")..."

    local actual_checksum
    actual_checksum=$(calculate_checksum "$file")

    if [ "$actual_checksum" = "$expected_checksum" ]; then
        log_success "Checksum verification passed"
        return 0
    else
        log_error "Checksum verification failed"
        log_error "Expected: $expected_checksum"
        log_error "Actual:   $actual_checksum"
        return 1
    fi
}

# Download file with progress indicator
download_file() {
    local url="$1"
    local output="$2"

    log_info "Downloading from: $url"
    log_info "Saving to: $output"

    # Create parent directory if it doesn't exist
    mkdir -p "$(dirname "$output")"

    # Download with progress bar
    if curl -# -L -f -o "$output" "$url"; then
        log_success "Download completed"
        return 0
    else
        log_error "Download failed"
        return 1
    fi
}

# Extract tarball
extract_archive() {
    local archive="$1"
    local output_dir="$2"

    log_info "Extracting archive to: $output_dir"

    mkdir -p "$output_dir"

    if tar -xzf "$archive" -C "$output_dir"; then
        log_success "Extraction completed"
        return 0
    else
        log_error "Extraction failed"
        return 1
    fi
}

# Download and install a single model
download_model() {
    local model_name="$1"

    log_info "=================================================="
    log_info "Processing model: $model_name"
    log_info "=================================================="

    # Get model info from registry
    local model_info
    model_info=$(get_model_info "$model_name")

    if [ -z "$model_info" ]; then
        log_error "Model not found in registry: $model_name"
        return 1
    fi

    # Extract model properties
    local download_url
    local checksum
    local version
    local size_mb
    local status

    download_url=$(echo "$model_info" | jq -r '.download_url')
    checksum=$(echo "$model_info" | jq -r '.checksum')
    version=$(echo "$model_info" | jq -r '.version')
    size_mb=$(echo "$model_info" | jq -r '.size_mb')
    status=$(echo "$model_info" | jq -r '.status')

    # Check if model is available
    if [ "$status" != "available" ]; then
        log_warning "Model status: $status (not available for download)"
        return 1
    fi

    # Setup paths
    local model_dir="${OUTPUT_DIR}/${model_name}"
    local archive_file="${OUTPUT_DIR}/${model_name}.tar.gz"

    # Check if model already exists
    if [ -d "$model_dir" ] && [ "$FORCE_DOWNLOAD" = false ]; then
        log_info "Model already exists: $model_dir"

        # Verify existing model if checksum available
        if [ -f "${model_dir}/model.onnx" ] && [ "$checksum" != "null" ]; then
            log_info "Verifying existing model..."
            if verify_checksum "${model_dir}/model.onnx" "$checksum"; then
                log_success "Model verified and up to date"
                return 0
            else
                log_warning "Checksum mismatch, re-downloading..."
                rm -rf "$model_dir"
            fi
        else
            log_info "Use --force to re-download"
            return 0
        fi
    fi

    # Download model archive
    if [ "$VERIFY_ONLY" = true ]; then
        log_info "Verify-only mode, skipping download"
        return 1
    fi

    log_info "Model version: $version"
    log_info "Model size: ${size_mb}MB"

    if ! download_file "$download_url" "$archive_file"; then
        return 1
    fi

    # Verify archive checksum
    if [ "$checksum" != "null" ]; then
        if ! verify_checksum "$archive_file" "$checksum"; then
            rm -f "$archive_file"
            return 1
        fi
    fi

    # Extract archive
    if ! extract_archive "$archive_file" "$OUTPUT_DIR"; then
        rm -f "$archive_file"
        return 1
    fi

    # Cleanup archive
    rm -f "$archive_file"

    # Verify extracted model structure
    if [ ! -f "${model_dir}/model.onnx" ]; then
        log_error "Model file not found after extraction: ${model_dir}/model.onnx"
        return 1
    fi

    if [ ! -d "${model_dir}/tokenizer" ]; then
        log_warning "Tokenizer directory not found: ${model_dir}/tokenizer"
    fi

    log_success "Model installed successfully: $model_dir"
    return 0
}

# Download all models from registry
download_all_models() {
    log_info "Downloading all models from registry..."

    local model_names
    model_names=$(jq -r '.models[].name' "$REGISTRY_FILE")

    local total=0
    local success=0
    local failed=0

    while IFS= read -r model_name; do
        total=$((total + 1))

        if download_model "$model_name"; then
            success=$((success + 1))
        else
            failed=$((failed + 1))
        fi

        echo ""
    done <<< "$model_names"

    # Print summary
    log_info "=================================================="
    log_info "Download Summary"
    log_info "=================================================="
    log_info "Total models: $total"
    log_success "Successful: $success"
    if [ $failed -gt 0 ]; then
        log_error "Failed: $failed"
    fi
    log_info "=================================================="

    return 0
}

# Verify all existing models
verify_all_models() {
    log_info "Verifying all existing models..."

    if [ ! -d "$OUTPUT_DIR" ]; then
        log_error "Output directory does not exist: $OUTPUT_DIR"
        return 1
    fi

    local verified=0
    local failed=0

    for model_dir in "$OUTPUT_DIR"/*; do
        if [ ! -d "$model_dir" ]; then
            continue
        fi

        local model_name
        model_name=$(basename "$model_dir")

        log_info "Verifying: $model_name"

        if [ -f "${model_dir}/model.onnx" ]; then
            local model_info
            model_info=$(get_model_info "$model_name")

            if [ -n "$model_info" ]; then
                local checksum
                checksum=$(echo "$model_info" | jq -r '.checksum')

                if [ "$checksum" != "null" ]; then
                    if verify_checksum "${model_dir}/model.onnx" "$checksum"; then
                        verified=$((verified + 1))
                    else
                        failed=$((failed + 1))
                    fi
                else
                    log_warning "No checksum available for: $model_name"
                fi
            else
                log_warning "Model not found in registry: $model_name"
            fi
        else
            log_error "Model file not found: ${model_dir}/model.onnx"
            failed=$((failed + 1))
        fi

        echo ""
    done

    log_info "=================================================="
    log_info "Verification Summary"
    log_info "=================================================="
    log_success "Verified: $verified"
    if [ $failed -gt 0 ]; then
        log_error "Failed: $failed"
    fi
    log_info "=================================================="

    return 0
}

# Main function
main() {
    echo ""
    log_info "LLM Shield Model Download Script"
    log_info "=================================================="

    # Parse arguments
    parse_args "$@"

    # Check requirements
    check_requirements

    # Load registry
    load_registry

    # List models if requested
    if [ "$LIST_MODELS" = true ]; then
        list_available_models
    fi

    # Create output directory
    mkdir -p "$OUTPUT_DIR"
    log_info "Output directory: $OUTPUT_DIR"
    echo ""

    # Execute requested operation
    if [ "$VERIFY_ONLY" = true ]; then
        verify_all_models
    elif [ -n "$MODEL_NAME" ]; then
        download_model "$MODEL_NAME"
    else
        download_all_models
    fi

    echo ""
    log_success "Operation completed!"
    echo ""
}

# Run main function
main "$@"
