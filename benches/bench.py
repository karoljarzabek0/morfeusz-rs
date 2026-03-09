import subprocess
import time
import os
import sys

# Ensure stdout handles UTF-8 correctly
if sys.stdout.encoding != 'utf-8':
    import io
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')

WORDS = ["dom", "domowi", "Aalborgach", "aaronowa", "bankowi", "kobieta", "robić", "pisać", "szkoła", "polska"] * 100

def bench_morfeusz():
    start = time.time()
    process = subprocess.Popen(['..\\vendor\\morfeusz\\morfeusz_analyzer.exe'], 
                             stdin=subprocess.PIPE, 
                             stdout=subprocess.PIPE, 
                             stderr=subprocess.PIPE, 
                             text=True, 
                             encoding='utf-8')
    input_text = "\n".join(WORDS) + "\n"
    stdout, stderr = process.communicate(input=input_text)
    end = time.time()
    return end - start

def bench_rust():
    # Pre-build to ensure we don't benchmark compilation
    subprocess.run(['cargo', 'build', '--release', '--bin', 'morfeusz-rs'], capture_output=True, cwd='..')
    
    start = time.time()
    process = subprocess.Popen(['..\\target\\release\\morfeusz-rs.exe'], 
                             stdin=subprocess.PIPE, 
                             stdout=subprocess.PIPE, 
                             stderr=subprocess.PIPE, 
                             text=True, 
                             encoding='utf-8')
    input_text = "\n".join(WORDS) + "\n"
    stdout, stderr = process.communicate(input=input_text)
    end = time.time()
    return end - start

print(f"Starting benchmark with {len(WORDS)} lookups...")

t_m = bench_morfeusz()
print(f"Morfeusz Analyzer: {t_m:.4f}s")

t_r = bench_rust()
print(f"Rust Analyzer:     {t_r:.4f}s")

if t_r > 0:
    print(f"Speedup: {t_m/t_r:.2f}x")
else:
    print("Rust was too fast to measure accurately.")
