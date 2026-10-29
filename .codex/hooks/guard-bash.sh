#!/usr/bin/env bash
# PreToolUse (Bash): block destructive commands outside the repo and commands that print secrets.
# A heuristic net, not a sandbox: permissions in settings.json are the real boundary.
set -uo pipefail
cmd="$(jq -r '.tool_input.command // empty')"
[[ -n "$cmd" ]] || exit 0
root="${CLAUDE_PROJECT_DIR:-$(pwd)}"
deny() { echo "Blocked: $1" >&2; exit 2; }

# Recursive or forced deletes aimed at /, ~, $HOME, parent directories or absolute paths
# other than the repository and /tmp.
sep='(^|[;&|[:space:]])'
rm_re="${sep}(sudo[[:space:]]+)?rm[[:space:]]+-[[:alnum:]-]*[rRf]"
if [[ "$cmd" =~ $rm_re ]]; then
  # Inspect every word after the first rm; quotes are stripped so "~" and '/' still match.
  read -ra words <<<"${cmd#*rm }"
  for target in "${words[@]}"; do
    target="${target//[\"\']/}"
    case "$target" in
      -*) ;;
      "$root" | "$root"/* | /tmp/*) ;;
      '&&' | '||' | ';' | '|') break ;;
      / | /* | '~' | '~/'* | '$HOME'* | '${HOME}'* | .. | ../* | */..*)
        deny "recursive delete outside the repository ($target)." ;;
    esac
  done
fi

# Commands that dump the environment, secret variables or .env files.
env_dump="${sep}(env|printenv|set|export[[:space:]]+-p)[[:space:]]*($|[;&|])"
printenv_var="${sep}printenv[[:space:]]"
secret_var='\$\{?[A-Za-z_]*(KEY|TOKEN|SECRET|PASSWORD|PASSWD)'
env_file="${sep}(cat|less|more|head|tail|bat|grep|rg|sed|awk|strings|xxd|cp|base64)[[:space:]][^;|&]*\.env([[:space:]]|$|\.(local|prod|production|dev|development|test))"
proc_env='/proc/[^[:space:]]*/environ'
# Match line by line so heredoc bodies (for example a .gitignore listing .env) do not trigger.
while IFS= read -r line; do
  for re in "$env_dump" "$printenv_var" "$secret_var" "$env_file" "$proc_env"; do
    [[ "$line" =~ $re ]] && deny "this command could print secrets from the environment or .env files."
  done
done <<<"$cmd"
exit 0
