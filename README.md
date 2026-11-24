# panbahar

Panbahar is a terminal blog editor that writes React components as you type.

## Usage

```bash
cargo run
```

Choose a `.tsx` or `.jsx` target from the terminal, then write directly in the editor.
Panbahar stores your source text separately inside the target file and reloads that source on the next session.
Only your blog text appears in the editor; generated JSX/functions stay in the React file.

```bash
cargo run -- blog.tsx
```

Press `Esc` to exit. Press `Ctrl-S` to explicitly save.
