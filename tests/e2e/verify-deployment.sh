#!/usr/bin/env bash
# Verify a deployed SAM project by hitting API endpoints
#
# Usage:
#   bash tests/e2e/verify-deployment.sh <project-dir> [project-type]
#   bash tests/e2e/verify-deployment.sh --help
#
# project-type: rest-api-sam | api-dynamodb-sam (auto-detected from dir name)
#
# Reads deploy-output.toml from project dir for API URL.
# Writes verify-output.toml with results.

set -euo pipefail

if [[ "${1:-}" == "--help" || -z "${1:-}" ]]; then
  echo "Usage: bash verify-deployment.sh <project-dir> [project-type]"
  echo ""
  echo "Verifies deployed API endpoints. Reads deploy-output.toml for API URL."
  echo "Project types: rest-api-sam, api-dynamodb-sam"
  exit 0
fi

PROJECT_DIR="$1"
PROJECT_TYPE="${2:-$(basename "$PROJECT_DIR")}"
DEPLOY_OUTPUT="$PROJECT_DIR/deploy-output.toml"
VERIFY_OUTPUT="$PROJECT_DIR/verify-output.toml"

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

if [[ ! -f "$DEPLOY_OUTPUT" ]]; then
  echo -e "${RED}No deploy-output.toml found${NC}"
  exit 1
fi

API_URL=$(grep "^api_url" "$DEPLOY_OUTPUT" | sed 's/.*= *"//' | tr -d '"')
if [[ -z "$API_URL" ]]; then
  echo -e "${RED}No API URL in deploy-output.toml${NC}"
  exit 1
fi

# Remove trailing slash
API_URL="${API_URL%/}"

PASS=0
FAIL=0
FAILURES=""

# Helper: check an endpoint
check_endpoint() {
  local method="$1" path="$2" expected_status="$3" body="${4:-}" label="${5:-$method $path}"
  local url="${API_URL}${path}"
  local args=(-s -o /tmp/konductor-verify-body -w "%{http_code}" -X "$method")
  [[ -n "$body" ]] && args+=(-H "Content-Type: application/json" -d "$body")

  local status
  status=$(curl "${args[@]}" "$url" 2>/dev/null || echo "000")

  if [[ "$status" == "$expected_status" ]]; then
    echo -e "  ${GREEN}✓ $label → $status${NC}"
    PASS=$((PASS + 1))
  else
    echo -e "  ${RED}✗ $label → $status (expected $expected_status)${NC}"
    FAIL=$((FAIL + 1))
    FAILURES="${FAILURES}${label}: got $status expected $expected_status; "
  fi
}

echo "Verifying: $PROJECT_TYPE"
echo "API URL: $API_URL"
echo ""

case "$PROJECT_TYPE" in
  rest-api-sam)
    # POST /items
    check_endpoint POST "/items" "201" '{"name":"Widget","price":9.99}' "POST /items (create)"
    # GET /items
    check_endpoint GET "/items" "200" "" "GET /items (list)"
    # POST to get an ID, then GET/DELETE it
    ITEM_BODY=$(curl -s -X POST -H "Content-Type: application/json" -d '{"name":"Test","price":1.0}' "${API_URL}/items" 2>/dev/null || echo "{}")
    ITEM_ID=$(echo "$ITEM_BODY" | python3 -c "import json,sys; print(json.load(sys.stdin).get('id',''))" 2>/dev/null || echo "")
    if [[ -n "$ITEM_ID" ]]; then
      check_endpoint GET "/items/$ITEM_ID" "200" "" "GET /items/{id}"
      check_endpoint DELETE "/items/$ITEM_ID" "200" "" "DELETE /items/{id}"
    else
      echo -e "  ${RED}✗ Could not extract item ID for GET/DELETE tests${NC}"
      FAIL=$((FAIL + 2))
    fi
    # 404 check
    check_endpoint GET "/items/nonexistent-id" "404" "" "GET /items/{bad-id} (404)"
    ;;

  api-dynamodb-sam)
    # POST /tasks
    check_endpoint POST "/tasks" "201" '{"title":"Buy milk","status":"pending"}' "POST /tasks (create)"
    # GET /tasks
    check_endpoint GET "/tasks" "200" "" "GET /tasks (list)"
    # POST to get an ID, then GET/PUT/DELETE
    TASK_BODY=$(curl -s -X POST -H "Content-Type: application/json" -d '{"title":"Test task"}' "${API_URL}/tasks" 2>/dev/null || echo "{}")
    TASK_ID=$(echo "$TASK_BODY" | python3 -c "import json,sys; print(json.load(sys.stdin).get('id',''))" 2>/dev/null || echo "")
    if [[ -n "$TASK_ID" ]]; then
      check_endpoint GET "/tasks/$TASK_ID" "200" "" "GET /tasks/{id}"
      check_endpoint PUT "/tasks/$TASK_ID" "200" '{"title":"Updated","status":"done"}' "PUT /tasks/{id}"
      check_endpoint DELETE "/tasks/$TASK_ID" "200" "" "DELETE /tasks/{id}"
    else
      echo -e "  ${RED}✗ Could not extract task ID for GET/PUT/DELETE tests${NC}"
      FAIL=$((FAIL + 3))
    fi
    # 404 check
    check_endpoint GET "/tasks/nonexistent-id" "404" "" "GET /tasks/{bad-id} (404)"
    ;;

  *)
    echo -e "${RED}Unknown project type: $PROJECT_TYPE${NC}"
    exit 1
    ;;
esac

echo ""
echo "Results: $PASS passed, $FAIL failed"

cat > "$VERIFY_OUTPUT" << EOF
verify_pass = $([[ $FAIL -eq 0 ]] && echo "true" || echo "false")
checks_total = $((PASS + FAIL))
checks_passed = $PASS
checks_failed = $FAIL
failures = "$FAILURES"
EOF

[[ $FAIL -eq 0 ]] && exit 0 || exit 1
