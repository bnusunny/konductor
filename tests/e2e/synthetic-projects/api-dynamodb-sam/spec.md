# Synthetic Project: API + DynamoDB with SAM

## Description
Build a serverless REST API using AWS Lambda, API Gateway, and DynamoDB, deployed with AWS SAM. The API manages a task list with full CRUD operations persisted to DynamoDB.

## Endpoints
```
GET    /tasks          — List all tasks
POST   /tasks          — Create a new task (JSON body: {"title": "...", "status": "pending"})
GET    /tasks/{id}     — Get task by ID
PUT    /tasks/{id}     — Update task (JSON body: {"title": "...", "status": "done"})
DELETE /tasks/{id}     — Delete task by ID
```

## Response Format
```json
// GET /tasks
{"tasks": [{"id": "uuid", "title": "Buy milk", "status": "pending", "created_at": "ISO8601"}]}

// POST /tasks (201 Created)
{"id": "uuid", "title": "Buy milk", "status": "pending", "created_at": "ISO8601"}

// PUT /tasks/{id} (200)
{"id": "uuid", "title": "Buy milk", "status": "done", "created_at": "ISO8601"}

// GET /tasks/{id} (404)
{"error": "Task not found"}

// DELETE /tasks/{id} (200)
{"message": "Task deleted"}
```

## DynamoDB Table Schema
- Table name: Tasks (parameterized via SAM)
- Partition key: `id` (String)
- Attributes: `title` (String), `status` (String), `created_at` (String)

## Requirements
- REQ-01: Lambda function handles GET/POST/PUT/DELETE for /tasks resource
- REQ-02: POST validates request body (title required, status defaults to "pending")
- REQ-03: PUT validates request body and returns 404 for non-existent tasks
- REQ-04: All data persisted to DynamoDB table
- REQ-05: SAM template defines Lambda, API Gateway, and DynamoDB table
- REQ-06: SAM template passes `sam validate`
- REQ-07: Lambda has IAM permissions for DynamoDB CRUD operations

## Expected Files
- `app.py` — Lambda handler with routing and DynamoDB operations
- `test_app.py` — pytest test suite using moto for DynamoDB mocking
- `template.yaml` — SAM template with DynamoDB resource
- `requirements.txt` — Python dependencies (boto3 for runtime, moto for tests)

## SAM Template Requirements
The `template.yaml` must include:
- `AWSTemplateFormatVersion: '2010-09-09'`
- `Transform: AWS::Serverless-2016-10-31`
- `AWS::Serverless::Function` resource with:
  - Runtime: python3.12
  - Handler: app.lambda_handler
  - Environment variable for table name
  - Policies: DynamoDBCrudPolicy referencing the table
  - Events with Api type for each endpoint
- `AWS::DynamoDB::Table` or `AWS::Serverless::SimpleTable` resource
- `Outputs` section with API endpoint URL

## Success Criteria
- `sam validate` passes
- `ruff check app.py test_app.py` passes with no errors
- `pytest test_app.py -v` passes all tests
- `pytest test_app.py` discovers at least 10 test cases
- Lambda handler returns correct status codes (200, 201, 400, 404)
- DynamoDB operations use boto3 resource or client correctly

## Test Suite Requirements
Tests must use moto to mock DynamoDB. Tests must cover:
- GET /tasks returns empty list initially
- POST /tasks creates task with generated ID and created_at
- POST /tasks with missing title returns 400
- POST /tasks defaults status to "pending"
- GET /tasks/{id} returns task from DynamoDB
- GET /tasks/{id} with bad ID returns 404
- PUT /tasks/{id} updates task in DynamoDB
- PUT /tasks/{id} with bad ID returns 404
- DELETE /tasks/{id} removes task from DynamoDB
- DELETE /tasks/{id} with bad ID returns 404

## Tech Stack
- Python 3.12 (Lambda runtime)
- AWS SAM for IaC
- DynamoDB for persistence
- boto3 for AWS SDK
- moto for DynamoDB mocking in tests
- pytest for testing
- ruff for linting
