"""
FastAPI integration example for LLM Shield.

This demonstrates how to use LLM Shield scanners in a FastAPI application
for real-time input/output scanning.
"""

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from llm_shield import BanSubstrings, Secrets, Sensitive, Vault
from typing import Optional

# Initialize FastAPI app
app = FastAPI(title="LLM Shield FastAPI Example")

# Initialize scanners (reuse across requests)
input_scanners = {
    "ban_substrings": BanSubstrings(
        substrings=["spam", "scam", "phishing"],
        case_sensitive=False,
        redact=True
    ),
    "secrets": Secrets(redact=True),
}

output_scanners = {
    "sensitive": Sensitive(),
}


# Request/Response models
class ScanRequest(BaseModel):
    text: str
    user_id: Optional[str] = None


class ScanResponse(BaseModel):
    sanitized_text: str
    is_valid: bool
    risk_score: float
    detected_issues: list[str]


class ChatRequest(BaseModel):
    prompt: str
    user_id: Optional[str] = None


class ChatResponse(BaseModel):
    response: str
    prompt_valid: bool
    response_valid: bool


@app.post("/scan/input", response_model=ScanResponse)
async def scan_input(request: ScanRequest):
    """
    Scan user input for security issues.

    This endpoint runs multiple input scanners and returns aggregated results.
    """
    vault = Vault()

    # Add user context to vault
    if request.user_id:
        vault.set("user_id", request.user_id)

    detected_issues = []
    max_risk_score = 0.0
    sanitized_text = request.text

    # Run all input scanners
    for scanner_name, scanner in input_scanners.items():
        result = scanner.scan(request.text, vault)

        if not result['is_valid']:
            detected_issues.append(scanner_name)
            max_risk_score = max(max_risk_score, result['risk_score'])
            sanitized_text = result['sanitized_input']

    return ScanResponse(
        sanitized_text=sanitized_text,
        is_valid=len(detected_issues) == 0,
        risk_score=max_risk_score,
        detected_issues=detected_issues
    )


@app.post("/scan/output", response_model=ScanResponse)
async def scan_output(prompt: str, output: str):
    """
    Scan LLM output for PII and other sensitive information.
    """
    vault = Vault()
    vault.set("prompt", prompt)

    detected_issues = []
    max_risk_score = 0.0

    # Run output scanners
    for scanner_name, scanner in output_scanners.items():
        result = scanner.scan_output(prompt, output, vault)

        if not result['is_valid']:
            detected_issues.append(scanner_name)
            max_risk_score = max(max_risk_score, result['risk_score'])

    return ScanResponse(
        sanitized_text=output,  # Could sanitize based on scanner results
        is_valid=len(detected_issues) == 0,
        risk_score=max_risk_score,
        detected_issues=detected_issues
    )


@app.post("/chat", response_model=ChatResponse)
async def chat(request: ChatRequest):
    """
    Complete chat flow with input and output scanning.

    This demonstrates the full pipeline:
    1. Scan user prompt
    2. If valid, generate LLM response (mocked here)
    3. Scan LLM response
    4. Return sanitized results
    """
    vault = Vault()

    # Scan user prompt
    prompt_valid = True
    for scanner in input_scanners.values():
        result = scanner.scan(request.prompt, vault)
        if not result['is_valid']:
            prompt_valid = False
            break

    if not prompt_valid:
        raise HTTPException(
            status_code=400,
            detail="Input contains prohibited content"
        )

    # Mock LLM response (in production, call actual LLM here)
    llm_response = f"This is a response to: {request.prompt}"

    # Scan LLM response
    response_valid = True
    sanitized_response = llm_response

    for scanner in output_scanners.values():
        result = scanner.scan_output(request.prompt, llm_response, vault)
        if not result['is_valid']:
            response_valid = False
            sanitized_response = result['sanitized_input']

    return ChatResponse(
        response=sanitized_response,
        prompt_valid=prompt_valid,
        response_valid=response_valid
    )


@app.get("/health")
async def health_check():
    """Health check endpoint."""
    return {
        "status": "healthy",
        "scanners": {
            "input": list(input_scanners.keys()),
            "output": list(output_scanners.keys()),
        }
    }


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)

"""
Usage:
    # Run server
    python fastapi_integration.py

    # Test endpoints
    curl -X POST http://localhost:8000/scan/input \
        -H "Content-Type: application/json" \
        -d '{"text": "This is a test message"}'

    curl -X POST http://localhost:8000/chat \
        -H "Content-Type: application/json" \
        -d '{"prompt": "What is the weather today?"}'
"""
