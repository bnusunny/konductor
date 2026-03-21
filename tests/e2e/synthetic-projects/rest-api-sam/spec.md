# Synthetic Project: REST API with SAM

## Description
Build a serverless REST API using AWS Lambda and API Gateway, deployed with AWS SAM. The API manages a list of items with CRUD operations. Data is stored in-memory (no database).

## Endpoints
```
GET    /items          — List all items
POST   /items          — Create a new item (JSON body: {"name": "...", "price": 0.0})
GET    /items/{id}     — Get item by ID
DELETE /items/{id}     — Delete item by ID
```

## Response Format
```json
// GET /items
{"items": [{"id": "1", "name": "Widget", "price": 9.99}]}

// POST /items (201 Created)
{"id": "generated-uuid", "name": "Widget", "price": 9.99}

// GET /items/{id}
{"id": "1", "name": "Widget", "price": 9.99}

// GET /items/{id} (404)
{"error": "Item not found"}

// DELETE /items/{id} (200)
{"message": "Item deleted"}
```

## Requirements
- REQ-01: Lambda function handles GET/POST/DELETE for /items resource
- REQ-02: POST validates request body (name required, price must be numeric)
- REQ-03: GET /items/{id} returns 404 for non-existent items
- REQ-04: SAM template defines Lambda function and API Gateway
- REQ-05: SAM template passes `sam validate`

## Expected Files
- `app.py` — Lambda handler with routing logic
- `test_app.py` — pytest test suite for handler
- `template.yaml` — SAM template
- `requirements.txt` — Python dependencies (empty or minimal)

## SAM Template Requirements
The `template.yaml` must include:
- `AWSTemplateFormatVersion: '2010-09-09'`
- `Transform: AWS::Serverless-2016-10-31`
- `AWS::Serverless::Function` resource with:
  - Runtime: python3.12
  - Handler: app.lambda_handler
  - Events with Api type for each endpoint
- `Outputs` section with API endpoint URL

## Success Criteria
- `sam validate` passes
- `ruff check app.py test_app.py` passes with no errors
- `pytest test_app.py -v` passes all tests
- `pytest test_app.py` discovers at least 8 test cases
- Lambda handler returns correct status codes (200, 201, 400, 404)

## Test Suite Requirements
Tests must cover:
- GET /items returns empty list initially
- POST /items creates item and returns 201
- POST /items with missing name returns 400
- GET /items/{id} returns item
- GET /items/{id} with bad ID returns 404
- DELETE /items/{id} removes item
- DELETE /items/{id} with bad ID returns 404
- POST /items with invalid price returns 400

## Tech Stack
- Python 3.12 (Lambda runtime)
- AWS SAM for IaC
- pytest for testing
- ruff for linting
