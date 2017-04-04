import os
import sys
import subprocess
from subprocess import PIPE
from collections import namedtuple

def LOCAL(*path):
    return os.path.join(os.path.dirname(__file__), *path)

TEST_DIR = LOCAL("maltests")

STEPS = {
    "step0": "step0_repl",
    "step1": "step1_read_print",
    "step2": "step2_eval",
    "step3": "step3_env",
}

def get_step(step: str) -> str:
    step_name = STEPS.get(step)
    if step_name is None:
        raise Exception("Step {!r} not found!".format(step))
    return step_name


Test = namedtuple("Test", ["name", "cases", "type"])
TestCase = namedtuple("TestCase", ["input_lines", "expected_output", "should_fail"])

class TestType:
    Mandatory = "Mandatory"
    Deferred = "Deferred"
    Optional = "Optional"


def build_rust(step_name):
    cmd = ["cargo", "build", "--bin", step_name]
    subprocess.run(cmd, check=True)


def rust_cmd(step_name):
    EXEPATH = LOCAL("target", "debug", step_name)
    return [EXEPATH]


def run_tests(tests, run_cmd):
    passed = []
    mandatory_failed = []
    deferrable_failed = []
    optional_failed = []
    
    for i, test in enumerate(tests):
        (test_input, expected_output, testtype) = test
        cmd = run_cmd + [test_input]
        res = subprocess.run(cmd, stderr=PIPE, stdout=PIPE, universal_newlines=True)
        
        if res.returncode != 0:
            if testtype != TestType.ShouldFail:
                print("{} ): ERROR! : {}".format(i, test_input))
                print("==== Error output ====\n{}".format(res.stderr))
                if testtype == TestType.Mandatory:
                    mandatory_failed.append(i)
                elif testtype == TestType.Deferred:
                    deferrable_failed.append(i)
                elif testtype == TestType.Optional:
                    optional_failed.append(i)
            else:
                print("{} ): PASSED! (by failing) : {!r}".format(i, test_input))
        else:
            output = res.stdout.rstrip() # strip to remove newline
            #print("Stdout: {!r}".format(output))
            if output == expected_output:
                print("{} ): PASSED! : {}".format(i, test_input))
                passed.append(i)
            else:
                print("{} ): FAILED! : {}".format(i, test_input))
                print("Input:    {!r}\n".format(test_input))
                print("Expected: {!r}\n".format(expected_output))
                print("Got:      {!r}\n".format(output))
                if testtype == TestType.Mandatory:
                    mandatory_failed.append(i)
                elif testtype == TestType.Deferred:
                    deferrable_failed.append(i)
                elif testtype == TestType.Optional:
                    optional_failed.append(i)
    
    return (passed, mandatory_failed, deferrable_failed, optional_failed)


def print_results(passed, mandatory_failed, deferrable_failed, optional_failed):
    def fail_text(name, faillist):
        if faillist:
            text = " [{}]".format(", ".join(str(i) for i in faillist))
        else:
            text = ""
        return "  {} {} tests failed {}".format(len(faillist), name, text)
    
    verdict = "SUCCES" if not mandatory_failed else "FAILURE"
    if not mandatory_failed and not deferrable_failed and not optional_failed:
        verdict = "PERFECT"
    
    print("")
    print("Results:")
    print("  {} tests passed".format(len(passed)))
    print(fail_text("mandatory", mandatory_failed))
    print(fail_text("deferrable", deferrable_failed))
    print(fail_text("optional", optional_failed))
    print("")
    print("Verdict: {}!".format(verdict))
    
            

def load_tests(step_name):
    filepath = os.path.join(TEST_DIR, step_name+".mal")
    if not os.path.exists(filepath):
        raise FileNotFoundException("Could not find test file: {!r}".format(step_name+".mal"))
    with open(filepath, "r") as f:
        text = f.read()
    
    tests = []
    test_type = TestType.Mandatory
    
    test_name = "NO TEST NAME"
    cases = []
    case_input_lines = []
    def start_new_test(lineno):
        if cases:
            if case_input_lines:
                raise Exception("Line {}: New test start, but no output for previous test case")
            test = Test(test_name, cases.copy(), test_type)
            tests.append(test)
            cases.clear()
        
        
    for i, line in enumerate(text.splitlines()):
        line = line.strip()
        if line.startswith(";;"):
            start_new_test(i+1)
            test_name = line[2:].strip()
        
        if line == "" or line.isspace():
            continue
        
        if line.startswith(";>>>"):
            ll = line.lower()
            if "optional" in ll:
                test_type = TestType.Optional
            elif "deferrable" in ll:
                test_type = TestType.Deferred
            else:
                raise Exception("Line {}: Unknown parse directive: {!r}", i+1, line)
        
        elif line.startswith(";=>"):
            #print("{}: OUTPUT: {}".format(i+1, line))
            if case_input_lines:
                output = line[3:]
                case = TestCase(case_input_lines.copy(), output, False)
                cases.append(case)
                case_input_lines.clear()
            else:
                raise Exception("Line {}: Found output line with no input".format(i+1))
        
        elif line.startswith("; expected"):
            if not case_input_lines:
                raise Exception("Line {}: Found output line with no input".format(i+1))
            case = TestCase(case_input_lines.copy(), "", True)
            cases.append(case)
            case_input_lines.clear()
        
        elif line.startswith(";"):
            if "not found" in line:
                if not case_input_lines:
                    raise Exception("Line {}: Found output line with no input".format(i+1))
                case = TestCase(case_input_lines.copy(), "", True)
                cases.append(case)
                case_input_lines.clear()
            
            continue
        
        else:
            #print("{}: INPUT: {}".format(i+1, line))
            if not case_input_lines:
                case_input_lines.append(line)
            else:
                case_input_lines.append(line)
                print("WARN: Line {}: Found second input line in a row".format(i+1))
    
    if case_input_lines:
        raise Exception("END: No output found for last input line")
    
    start_new_test(i+1)
    
    return tests


def print_tests(tests):
    print("Tests:")
    for test in tests:
        print("TEST: {} ({})".format(test.name, test.type))
        print("")
        maxw = len(str(len(test.cases)))
        tag = "{{:<{}}}".format(maxw)
        for i, case in enumerate(test.cases, 1):
            itag = tag.format(i)
            ptag = tag.format(" ")
            #print(itag)
            for j, input_line in enumerate(case.input_lines):
                if j == 0:
                    prefix = itag
                else:
                    prefix = itag
                print("{}| user> {}".format(prefix, input_line))

            output = "<Error>" if case.should_fail else "-> " + case.expected_output
            print("{}| {}".format(itag, output))
            print("")
            #print("-"*(maxw+1))
        
    

def main(args=sys.argv[1:]):
    if not args:
        return print("Usage: python3 test.py <step>")
    from pprint import pprint
    step_name = get_step(args[0])
    tests = load_tests(step_name)
    print_tests(tests)
    
    #build_rust(step_name)
    #cmd = rust_cmd(step_name)
    #(passed, manfail, deffail, optfail) = run_tests(tests, cmd)
    #print_results(passed, manfail, deffail, optfail)


if __name__ == '__main__':
    main()

