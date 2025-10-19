import subprocess, shutil, socket, sys
from typing import Optional, TextIO

testfiles = ["happy_path.txt"]
process: subprocess.Popen = subprocess.Popen([shutil.which("cargo"), "run"], stdout=subprocess.PIPE, stderr=subprocess.PIPE) # type: ignore

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


def init_socket(address: str, port: int) -> Optional[TextIO]:
    
    sail_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sail_socket.connect((address, port))
    return sail_socket.makefile("rw")

def test(sail_fd: TextIO, testfile: TextIO):
    for line in testfile:
        print("test line: " + line.removesuffix("\n"))
        next_line = sanitize_newlines(line)
        if next_line.startswith("S:"):
            expected_response = next_line.replace("S: ", "")
            sail_response = sanitize_newlines(sail_fd.readline())
            print("sail response: " + sail_response)

            if sail_response != expected_response:
                print(f"Diff test failed: sail vs expected: \n{sail_response}\n{expected_response}")
                return -1
        elif next_line.startswith("C:"):
            sail_fd.write(next_line.replace("C: ", "").replace("\\r", "").replace("\\n", "") + "\r\n")
            sail_fd.flush()
    print("Completed test with no problems :)")

def sanitize_newlines(input: str) -> str:
    return input.replace("\r", "\\r").replace("\n", "\\n")

def run_tests():
    sail = start_sail()
    if sail is None:
        return None
    
    for testfile in testfiles:
        socket = init_socket(sail[0], sail[1])
        if socket is None:
            return None
        
        test(socket, open("saild/test/" + testfile))

try:
    run_tests()
finally:
    if process is not None:
        process.kill()