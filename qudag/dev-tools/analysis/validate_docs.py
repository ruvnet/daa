#!/usr/bin/env python3
"""
Simple script to validate documentation examples by checking syntax.
This doesn't guarantee the examples work but ensures basic syntax is correct.
"""

import os
import re
import subprocess
import sys

def find_rust_files(directory):
    """Find all Rust source files."""
    rust_files = []
    for root, dirs, files in os.walk(directory):
        # Skip target and hidden directories
        dirs[:] = [d for d in dirs if not d.startswith('.') and d != 'target']
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(os.path.join(root, file))
    return rust_files

def extract_doc_examples(file_path):
    """Extract Rust code examples from documentation comments."""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Pattern to match Rust code blocks in doc comments
    pattern = r'///.*?```rust\n(.*?)```'
    examples = re.findall(pattern, content, re.DOTALL | re.MULTILINE)
    
    # Also check for ```rust,ignore examples
    pattern_ignore = r'///.*?```rust,ignore\n(.*?)```'
    ignored_examples = re.findall(pattern_ignore, content, re.DOTALL | re.MULTILINE)
    
    return examples, ignored_examples

def validate_example_syntax(example_code):
    """Validate Rust example syntax using rustc --parse-only."""
    try:
        # Write example to temporary file
        with open('/tmp/example.rs', 'w') as f:
            f.write(example_code)
        
        # Try to parse with rustc
        result = subprocess.run(
            ['rustc', '--edition', '2021', '-Z', 'parse-only', '/tmp/example.rs'],
            capture_output=True,
            text=True
        )
        
        if result.returncode == 0:
            return True, None
        else:
            return False, result.stderr
    except Exception as e:
        return False, str(e)

def main():
    print("Validating documentation examples...")
    
    core_dir = os.path.join(os.getcwd(), 'core')
    if not os.path.exists(core_dir):
        print("Error: core directory not found")
        return 1
    
    rust_files = find_rust_files(core_dir)
    print(f"Found {len(rust_files)} Rust files")
    
    total_examples = 0
    valid_examples = 0
    ignored_examples = 0
    
    for file_path in rust_files:
        examples, ignored = extract_doc_examples(file_path)
        
        if examples or ignored:
            rel_path = os.path.relpath(file_path)
            print(f"\n{rel_path}:")
            
            for i, example in enumerate(examples):
                total_examples += 1
                is_valid, error = validate_example_syntax(example)
                
                if is_valid:
                    valid_examples += 1
                    print(f"  ✓ Example {i+1}")
                else:
                    print(f"  ✗ Example {i+1}: {error[:100]}...")
            
            if ignored:
                ignored_examples += len(ignored)
                print(f"  📝 {len(ignored)} ignored examples")
    
    print(f"\n=== Summary ===")
    print(f"Total examples: {total_examples}")
    print(f"Valid examples: {valid_examples}")
    print(f"Ignored examples: {ignored_examples}")
    print(f"Invalid examples: {total_examples - valid_examples}")
    
    if total_examples == 0:
        print("No documentation examples found!")
        return 1
    
    success_rate = (valid_examples / total_examples) * 100
    print(f"Success rate: {success_rate:.1f}%")
    
    return 0 if success_rate >= 80 else 1

if __name__ == '__main__':
    sys.exit(main())