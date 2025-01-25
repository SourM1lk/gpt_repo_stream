# GPT Repo Stream

Repo Stream prepares codebases for AI models by watching a designated path and automatically updating a structured output file for the entire codebase.

## How to Use
1. Clone the repo and build it:
```bash
git clone https://github.com/your-username/repo-stream.git
cd repo-stream
cargo build --release
```

2. Run it:
```bash
./target/release/gpt_repo_stream --repo /path/to/repo --output output.txt
```

3. That’s it! You’ll get an output.txt file containing your codebase in a format ready for AI.

## Command-Line Options
```
--repo (or -r): The repository you want to process (required).
--output (or -o): The name of the output file (default: output.txt).
```

## Excluding Files
You can use a .gptignore file in your repository to exclude files or directories. It works just like a .gitignore.

Example .gptignore:
```
node_modules/
target/
output.txt
```