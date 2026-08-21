# Agent Instructions

- `mainspec.md` is the product source of truth; consult relevant sections when needed, not as a mandatory full read for every task.
- Follow the active change under `openspec/changes/`.
- Do not implement outside the approved change scope.
- Keep architecture simple; avoid speculative abstractions.
- Stack: Tauri 2 + Rust + Vanilla TypeScript/HTML/CSS.
- Target: Windows 10/11 x64. Linux is development-only.
- Preserve existing working behavior, privacy, and fallback guarantees.