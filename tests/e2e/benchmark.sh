#!/usr/bin/env bash
# Benchmark harness for konductor E2E pipeline
# Runs each synthetic project independently and collects granular metrics
#
# Usage:
#   KONDUCTOR_E2E=1 bash tests/e2e/benchmark.sh
#   KONDUCTOR_E2E=1 bash tests/e2e/benchmark.sh --dry-run
#   KONDUCTOR_E2E=1 bash tests/e2e/benchmark.sh --project cli-calculator
#
# Output:
#   Per-project results in tests/e2e/results/{project}.toml
#   Combined summary in tests/e2e/last-results.toml

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
source "$SCRIPT_DIR/lib.sh"

DRY_RUN=false
FILTER_PROJECT=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=true; shift ;;
    --project) FILTER_PROJECT="$2"; shift 2 ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

if [[ "${KONDUCTOR_E2E:-}" != "1" ]]; then
  echo "Skipping benchmark (set KONDUCTOR_E2E=1 to run)"
  exit 0
fi

if [[ "$DRY_RUN" == "false" ]] && ! command -v kiro-cli >/dev/null 2>&1; then
  echo "ERROR: kiro-cli not found in PATH"
  exit 1
fi

# Discover projects
PROJECTS=()
while IFS= read -r p; do
  if [[ -z "$FILTER_PROJECT" || "$p" == "$FILTER_PROJECT" ]]; then
    PROJECTS+=("$p")
  fi
done < <(list_synthetic_projects)

if [[ ${#PROJECTS[@]} -eq 0 ]]; then
  echo "ERROR: No synthetic projects found"
  exit 1
fi

echo -e "${CYAN}═══ Konductor Benchmark ═══${NC}"
echo "Projects: ${PROJECTS[*]}"
echo ""

if [[ "$DRY_RUN" == "true" ]]; then
  echo "Dry run — listing projects only:"
  for p in "${PROJECTS[@]}"; do
    echo "  - $p ($(cat "$SCRIPT_DIR/synthetic-projects/$p/spec.md" | head -1))"
  done
  exit 0
fi

# Clean and create results directory
RESULTS_DIR="$SCRIPT_DIR/results"
rm -rf "$RESULTS_DIR"
mkdir -p "$RESULTS_DIR"

OVERALL_START=$(date +%s)
PROJECT_RESULTS=()

for PROJECT in "${PROJECTS[@]}"; do
  echo -e "\n${CYAN}═══ Project: $PROJECT ═══${NC}"

  SPEC_FILE="$SCRIPT_DIR/synthetic-projects/$PROJECT/spec.md"
  SPEC_CONTENT=$(cat "$SPEC_FILE")
  RESULT_FILE="$RESULTS_DIR/$PROJECT.toml"

  cat > "$RESULT_FILE" << EOF
# Benchmark results for $PROJECT
# Generated: $(date -Iseconds)

[project]
name = "$PROJECT"
timestamp = "$(date -Iseconds)"

EOF

  # Setup isolated test directory
  setup_project_test_dir "$PROJECT"
  PROJECT_DIR="$TEST_DIR"

  # --- Time each pipeline step ---

  echo "[timing]" >> "$RESULT_FILE"

  for step in init plan exec verify; do
    case "$step" in
      init)
        MSG="Initialize this project with Konductor. Here is the project spec:

$SPEC_CONTENT

This is a small 1-phase project. Generate project.md, requirements.md, roadmap.md with a single phase.
Do NOT ask me questions — use the spec above to generate all documents.
Run the konductor-init skill."
        ;;
      plan)
        MSG="Plan phase 01 for execution. Research the ecosystem, create execution plans with tasks and acceptance criteria, and validate them. Run the konductor-plan skill."
        ;;
      exec)
        MSG="Execute the plans for the current phase. Spawn executor subagents to implement each plan following TDD workflow. Run the konductor-exec skill."
        ;;
      verify)
        MSG="Verify the current phase after execution. Validate that phase goals were achieved using the 3-level verification framework. Run the konductor-verify skill."
        ;;
    esac

    local_start=$(date +%s)
    log_step "$PROJECT / $step"
    (cd "$PROJECT_DIR" && konductor_chat "$MSG" > /dev/null 2>&1) || true
    local_end=$(date +%s)
    elapsed=$((local_end - local_start))
    echo "${step}_s = $elapsed" >> "$RESULT_FILE"
    echo -e "  ${GREEN}✓ ${elapsed}s${NC}"
  done

  PROJ_END=$(date +%s)
  echo "total_s = $((PROJ_END - local_start))" >> "$RESULT_FILE"

  # --- Collect artifact metrics ---

  echo "" >> "$RESULT_FILE"
  echo "[artifacts]" >> "$RESULT_FILE"

  plan_count=$(find "$PROJECT_DIR/.konductor/phases" -name "*.md" ! -name "*-summary.md" ! -name "verification.md" ! -name "research.md" 2>/dev/null | wc -l)
  summary_count=$(find "$PROJECT_DIR/.konductor/phases" -name "*-summary.md" 2>/dev/null | wc -l)
  python_files=$(find "$PROJECT_DIR" -maxdepth 3 -name "*.py" ! -path "*/.konductor/*" 2>/dev/null | wc -l)
  test_files=$(find "$PROJECT_DIR" -maxdepth 3 -name "test_*.py" ! -path "*/.konductor/*" 2>/dev/null | wc -l)
  has_sam=$([[ -f "$PROJECT_DIR/template.yaml" ]] && echo "true" || echo "false")

  echo "plan_count = $plan_count" >> "$RESULT_FILE"
  echo "summary_count = $summary_count" >> "$RESULT_FILE"
  echo "python_files = $python_files" >> "$RESULT_FILE"
  echo "test_files = $test_files" >> "$RESULT_FILE"
  echo "sam_template = $has_sam" >> "$RESULT_FILE"

  # --- Code quality checks ---

  echo "" >> "$RESULT_FILE"
  echo "[quality]" >> "$RESULT_FILE"

  run_lint "$PROJECT_DIR" "$RESULT_FILE"
  run_tests "$PROJECT_DIR" "$RESULT_FILE"
  check_sam_template "$PROJECT_DIR" "$RESULT_FILE"

  # --- Deployment cycle (SAM projects only) ---

  echo "" >> "$RESULT_FILE"
  echo "[deployment]" >> "$RESULT_FILE"
  run_deploy_cycle "$PROJECT_DIR" "$RESULT_FILE"

  # --- Cleanup ---

  if [[ "${KEEP_TEST_DIR:-}" != "1" ]]; then
    rm -rf "$PROJECT_DIR"
    TEST_DIR=""
  fi

  PROJECT_RESULTS+=("$RESULT_FILE")
  echo -e "${GREEN}Results: $RESULT_FILE${NC}"
done

OVERALL_END=$(date +%s)
OVERALL_ELAPSED=$((OVERALL_END - OVERALL_START))

# --- Write combined summary ---

SUMMARY_FILE="$SCRIPT_DIR/last-results.toml"
cat > "$SUMMARY_FILE" << EOF
# Konductor Benchmark Summary
# Generated: $(date -Iseconds)
# Total time: ${OVERALL_ELAPSED}s
# Projects: ${PROJECTS[*]}

[summary]
total_time_s = $OVERALL_ELAPSED
project_count = ${#PROJECTS[@]}
projects = [$(printf '"%s", ' "${PROJECTS[@]}" | sed 's/, $//')]

EOF

for rf in "${PROJECT_RESULTS[@]}"; do
  pname=$(basename "$rf" .toml)
  echo "# --- $pname ---" >> "$SUMMARY_FILE"
  cat "$rf" >> "$SUMMARY_FILE"
  echo "" >> "$SUMMARY_FILE"
done

echo -e "\n${CYAN}═══ Benchmark Complete ═══${NC}"
echo "Total time: ${OVERALL_ELAPSED}s"
echo "Projects: ${#PROJECTS[@]}"
echo "Results: $RESULTS_DIR/"
echo "Summary: $SUMMARY_FILE"
