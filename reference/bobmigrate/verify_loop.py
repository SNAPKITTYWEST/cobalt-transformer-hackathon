import subprocess
import sys
import os

MAX_RETRIES = 5

def call_llm(prompt):
    """
    TODO: Replace this mock function with your actual SDK call
    to IBM Watsonx, OpenAI, Anthropic, or local LLM.
    """
    print("\n[🤖 LLM is rewriting code...]")
    # Return the raw generated code here
    return ""

def run_verification_loop():
    print("Starting Autonomous Verification Pipeline...")

    # Ensure the directory exists
    os.makedirs("models", exist_ok=True)

    for attempt in range(1, MAX_RETRIES + 1):
        print(f"\n▶ Attempt {attempt} of {MAX_RETRIES}...")

        # Run standard Go tests
        result = subprocess.run(
            ["go", "test", "./models", "-v"],
            capture_output=True,
            text=True
        )

        # If exit code is 0, the code compiled and all tests passed
        if result.returncode == 0:
            print(result.stdout)
            print("✅ SUCCESS: All tests passed. Pipeline complete.")
            sys.exit(0)

        print("❌ FAILED: Bug detected. Initiating self-correction loop.")

        # Combine stdout and stderr to ensure compiler errors are caught
        error_output = result.stdout + "\n" + result.stderr
        print(f"Captured Error:\n{error_output}")

        # Read the broken code to send back to the AI
        try:
            with open("models/models.go", "r") as f:
                broken_code = f.read()
        except FileNotFoundError:
            broken_code = "// File not found"

        correction_prompt = f"""
        You are an expert Go Developer fixing a bug in a legacy migration.
        Your previous translation failed the test suite or failed to compile.

        ### CURRENT GO CODE:
        {broken_code}

        ### COMPILER / TEST ERROR:
        {error_output}

        ### TASK:
        Rewrite the entire `models.go` file to fix the bug.
        Maintain pure int64 math. Do not use float64.
        Output ONLY valid Go code. No markdown. No explanations.
        """

        fixed_code = call_llm(correction_prompt)

        # Write the AI's fix back to disk for the next loop iteration
        if fixed_code.strip():
            with open("models/models.go", "w") as f:
                f.write(fixed_code)

    print("\n⚠️ FATAL: Max retries reached. Human intervention required.")
    sys.exit(1)

if __name__ == "__main__":
    run_verification_loop()
