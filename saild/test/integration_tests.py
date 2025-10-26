import subprocess, shutil, socket, sys, argparse
from typing import Optional, TextIO

testfiles = ["happy_path.txt"]
process: subprocess.Popen = subprocess.Popen([shutil.which("cargo"), "run"], stdout=subprocess.PIPE, stderr=subprocess.PIPE) # type: ignore
is_error: bool = False

# returns a tuple of address and port
def start_sail() -> Optional[tuple[str, int]]:
    if process.stdout == None:
        print("failed to start sail process")
        return None

    first_bytes: bytes = process.stdout.readline()
    first_line: str = first_bytes.decode()
    if not first_line.startswith("saild started"):
        print("error initializing: " + first_line)
        return None
    
    socket_address = first_line.split(" ")[-1].split(":")
    return socket_address[0], int(socket_address[1])


def init_socket(address: str, port: int) -> TextIO:
    sail_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sail_socket.connect((address, port))
    return sail_socket.makefile("rw")

def test(sail_fd: TextIO, testfile_name: str, generate: bool) -> int:
    generated_lines: list[str] = []
    with open(testfile_name) as testfile:
        testlines = [line for line in testfile]
    for line in testlines:
        print("test line: " + line.removesuffix("\n"))
        next_line = sanitize_newlines(line)
        if next_line.startswith("S:"):
            expected_response = next_line.replace("S: ", "")
            sail_response = sanitize_newlines(sail_fd.readline())
            print("sail response: " + sail_response)

            if sail_response != expected_response:
                if generate:
                    generated_lines.append("S: " + unsanitize_newlines(sail_response).strip())
                    while len(generated_lines[-1]) >= 7 and generated_lines[-1][6] == '-':
                        generated_line = sail_fd.readline()
                        generated_lines.append("S: " + generated_line.strip())
                else:
                    print(f"Diff test failed: sail vs expected: \n{sail_response}\n{expected_response}")
                    return -1
            else:
                generated_lines.append("S: " + unsanitize_newlines(sail_response).strip())
        elif next_line.startswith("C:"):
            clean_line = next_line.replace("C: ", "").replace("\\r", "").replace("\\n", "")
            sail_fd.write(clean_line + "\r\n")
            sail_fd.flush()
            generated_lines.append("C: " + clean_line)
    if generate:
        generated_text = "\r\n".join(generated_lines) + "\r\n"
        print(f"Writing following text to {testfile_name}:\n")
        print(generated_text)
        with open(testfile_name, "w") as testfile:
            testfile.write(generated_text)
    print("Completed test with no problems :)")
    return 0

def sanitize_newlines(input: str) -> str:
    return input.replace("\r", "\\r").replace("\n", "\\n")
def unsanitize_newlines(input: str) -> str:
    return input.replace("\\r", "\r").replace("\\n", "\n")

def run_tests(generate: bool) -> Optional[int]:
    sail = start_sail()
    if sail is None:
        return None
    
    for testfile in testfiles:
        socket = init_socket(sail[0], sail[1])
        if socket is None:
            return None
        
        if test(socket, "saild/test/" + testfile, generate) != 0:
            return None
    return 0


parser = argparse.ArgumentParser()
parser.add_argument('-g', '--generate', action='store_true')
args = parser.parse_args()

try:
    if run_tests(args.generate) is None:
        is_error = True
finally:
    if process is not None:
        process.kill()
sys.exit(-1 if is_error else 0)