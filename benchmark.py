"""To be run from project root"""
from pathlib import Path
import subprocess
import time
from typing import Union

if __name__ == "__main__":
    subprocess.run(["cargo", "build", "--release"])

    def print_line(col1: str, col2: Union[str, float], col3: Union[str, float]):
        col2 = col2 if isinstance(col2, str) else f"{col2:.4f}"
        col3 = col3 if isinstance(col3, str) else f"{col3:.4f}"
        print(f"{col1: ^15} | {col2: ^15} | {col3: ^15}")
    
    print_line("benchmark", "calculator", "python")

    for benchmark_dir in (Path(__file__).parent / "benchmarks").iterdir():
        benchmark_files = list(benchmark_dir.iterdir())

        clc_file = next(iter([f for f in benchmark_files if f.suffix == ".clc"]))
        start = time.time()
        subprocess.run(["./target/release/calculator", str(clc_file)], capture_output=True)
        calculator_time = time.time() - start


        py_file = next(iter([f for f in benchmark_files if f.suffix == ".py"]))
        start = time.time()
        subprocess.run(["python", str(py_file)], capture_output=True)
        python_time = time.time() - start

        print_line(benchmark_dir.name, calculator_time, python_time)
