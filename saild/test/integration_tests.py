from __future__ import annotations
import subprocess, shutil, socket, sys, argparse
from dataclasses import dataclass
from typing import TextIO
from itertools import chain

testfiles = ["happy_path.txt", "abortive.txt"]
process: subprocess.Popen = subprocess.Popen([shutil.which("cargo"), "run"], stdout=subprocess.PIPE, stderr=subprocess.PIPE) # type: ignore
is_error: bool = True

@dataclass
class Command:
    command: list[str]
    response: list[str]

@dataclass
class TestCase:
    initial_response: str
    commands: list[Command]
    
    def validate_initial_response(self, server_response: str):
        if server_response != self.initial_response:
            raise Exception(f"Initial response differs - expected {self.initial_response.strip()}, got {server_response.strip()}")
    
    @staticmethod
    def parse_testlines(testlines: list[str]) -> TestCase:
        retval = TestCase(testlines[0][3:], [])
        
        i = 1
        while i < len(testlines):
            if not testlines[i].startswith("C: ") and not testlines[i].startswith("S: "):
                # invalid line
                i += 1
                continue

            command = Command([], [])
            while i < len(testlines) and testlines[i].startswith("C: "):
                command.command.append(testlines[i][3:].strip())
                i += 1
            while i < len(testlines) and testlines[i].startswith("S: "):
                command.response.append(testlines[i][3:].strip())
                i += 1
            retval.commands.append(command)
        return retval
    
    def unparse_testlines(self) -> str:
        lines = chain(
            ["S: " + self.initial_response], 
            *map(
                lambda command: chain(
                    map(lambda command_line: "C: " + command_line, command.command), 
                    map(lambda response_line: "S: " + response_line, command.response)
                ), 
                self.commands
            )
        )

        return "\r\n".join(lines) + "\r\n"


# returns a tuple of address and port
def start_sail() -> tuple[str, int]:
    if process.stdout is None:
        raise Exception("failed to start sail process")

    first_bytes: bytes = process.stdout.readline()
    first_line: str = first_bytes.decode()
    if not first_line.startswith("saild started"):
        raise Exception("error initializing: " + first_line)
    
    socket_address = first_line.split(" ")[-1].split(":")
    return socket_address[0], int(socket_address[1])


def init_socket(address: str, port: int) -> TextIO:
    sail_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sail_socket.connect((address, port))
    return sail_socket.makefile("rw")


def test(sail_fd: TextIO, testfile_name: str, generate: bool, codes_only: bool):
    with open(testfile_name) as testfile:
        testcase = TestCase.parse_testlines([line for line in testfile])

    if generate:
        testcase.initial_response = sail_fd.readline().strip()
    else:
        testcase.validate_initial_response(sail_fd.readline())

    for command in testcase.commands:
        for command_line in command.command:
            sail_fd.write(command_line + "\r\n")
        sail_fd.flush()

        sail_responses: list[str] = []
        while True:
            sail_response = sail_fd.readline()
            sail_responses.append(sail_response.strip())
            if not (len(sail_responses[-1]) >= 4 and sail_responses[-1][3] == '-'):
                break

        full_equals = lambda responses: responses[0] == responses[1]
        code_equals = lambda responses: responses[0][:3] == responses[1][:3]
        comparer = code_equals if codes_only else full_equals
        
        responses_equal = all(map(comparer, zip(sail_responses, command.response)))
        if generate:
            command.response = sail_responses
        elif not responses_equal:
            raise Exception(f"sail response ({sail_responses}) differs from expected response ({command.response})")
        
    if generate:
        generated_text = testcase.unparse_testlines()
        print(f"Writing following text to {testfile_name}:\n")
        print(generated_text)
        with open(testfile_name, "w") as testfile:
            testfile.write(generated_text)
    print(f"Completed test {testfile_name.split("/")[-1]} with no problems :)")

def run_tests(generate: bool, codes_only: bool):
    sail = start_sail()
    
    for testfile in testfiles:
        socket = init_socket(sail[0], sail[1])
        
        test(socket, "saild/test/" + testfile, generate, codes_only)


parser = argparse.ArgumentParser()
group = parser.add_mutually_exclusive_group()
group.add_argument('-g', '--generate', action='store_true')
group.add_argument('-c', '--codes-only', action='store_true')
args = parser.parse_args()

try:
    run_tests(args.generate, args.codes_only)
    is_error = False
finally:
    if process is not None:
        process.kill()
sys.exit(-1 if is_error else 0)