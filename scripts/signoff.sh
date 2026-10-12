#!/usr/bin/env bash
# Local CI and sign-off: runs the checks on each open pull request's head
# commit and posts the result to that commit as a `signoff` status, which
# GitHub shows on the pull request (and branch protection can require).
#
#     mise run signoff              every open pull request
#     mise run signoff 18 logos-sheen   just these (numbers or branch names)
#
# Each commit's files, from `git archive` and nothing else, are copied into one
# reused tree, target/ci/tree, keeping its build caches, so a stack of branches
# builds incrementally. There it runs what CI ran: npm ci, then mise run lint,
# test, build and size. The contract's time budgets assume a fast machine,
# which is why this runs here rather than on a shared runner. Logs are in
# target/ci/logs. It needs gh, logged in to the repository.
set -uo pipefail

root=$(git rev-parse --show-toplevel)
tree=$root/target/ci/tree
logs=$root/target/ci/logs
repo=$(gh repo view --json nameWithOwner --jq .nameWithOwner)
host=$(hostname -s)
mkdir -p "$tree" "$logs"

status() { # sha state description
  gh api -X POST "repos/$repo/statuses/$1" -f state="$2" -f context=signoff -f description="$3" >/dev/null </dev/null
}

wanted() { # number branch: true when no arguments were given, or one names it
  [ ${#args[@]} -eq 0 ] && return 0
  for a in ${args[@]+"${args[@]}"}; do
    if [ "$a" = "$1" ] || [ "$a" = "$2" ]; then return 0; fi
  done
  return 1
}

args=("$@")
passed=0
failed=()
while read -r number branch sha; do
  wanted "$number" "$branch" || continue
  log=$logs/$number-$branch.log
  printf '#%s %s (%s): ' "$number" "$branch" "${sha:0:7}"
  git cat-file -e "$sha^{commit}" 2>/dev/null || git fetch -q origin "$branch" </dev/null
  status "$sha" pending "local CI running on $host"
  scratch=$(mktemp -d)
  git archive "$sha" | tar -x -C "$scratch"
  # By content, not times: git archive dates every file to its commit, older
  # than the last branch's build, so cargo would keep that build. A file that
  # changed is written now and rebuilt; one that didn't keeps its time and cache.
  rsync -rlpD --checksum --delete --exclude /target --exclude node_modules --exclude elm-stuff "$scratch/" "$tree/"
  rm -rf "$scratch"
  if (
    cd "$tree" &&
      mise trust --quiet mise.toml &&
      npm ci --no-audit --no-fund &&
      mise run lint && mise run test && mise run build && mise run size
  ) >"$log" 2>&1 </dev/null; then
    status "$sha" success "mise run lint, test, build and size passed on $host"
    echo "signed off"
    passed=$((passed + 1))
  else
    status "$sha" failure "local CI failed on $host; see target/ci/logs"
    echo "FAILED, see $log"
    failed+=("#$number")
  fi
done < <(gh pr list --state open --limit 200 --json number,headRefName,headRefOid \
  --jq 'sort_by(.number)[] | "\(.number) \(.headRefName) \(.headRefOid)"')

if [ ${#failed[@]} -eq 0 ]; then
  echo "$passed signed off"
else
  echo "$passed signed off, failed: ${failed[*]}"
  exit 1
fi
