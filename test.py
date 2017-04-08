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
    "step4": "step4_if_fn_do",
}

def get_step(step: str) -> str:
    step_name = STEPS.get(step)
    if step_name is None:
        raise Exception("Step {!r} not found!".format(step))
    return step_name


Test = namedtuple("Test", ["name", "cases", "type", "should_fail"])
TestCase = namedtuple("TestCase", ["input_lines", "expected_output"])

class TestType:
    Mandatory = "Mandatory"
    Deferrable = "Deferrable"
    Optional = "Optional"


def build_rust(step_name):
    cmd = ["cargo", "build", "--bin", step_name]
    subprocess.run(cmd, check=True)


def rust_cmd(step_name):
    EXEPATH = LOCAL("target", "debug", step_name)
    return [EXEPATH]


TestFailure = namedtuple("TestFailure", ["test", "case_numbers"])

def run_tests(tests, run_cmd):
    passed = []
    failed = {
        TestType.Mandatory: [],
        TestType.Deferrable: [],
        TestType.Optional: [],
    }
    print("Running {} tests...".format(len(tests)))
    print("")
    for i, test in enumerate(tests):
        failtext = " <Should fail> " if test.should_fail else ""
        print(" {} ({}){} ".format(test.name, test.type, failtext).center(80, "="))
        print("")
        failed_cases = []
        
        # Run the exe with all the input lines
        cmd = run_cmd.copy()
        for case in test.cases:
            cmd.extend(case.input_lines)
        
        res = subprocess.run(cmd, stderr=PIPE, stdout=PIPE, universal_newlines=True)
        if res.returncode != 0:
            if not test.should_fail:
                # Even if the test 'errors' it's fine if the output is correct
                # See step3_env.mal ';; Check that error aborts def!'
                output = res.stdout.rstrip().splitlines()
                # show which cases passed and which one failed
            else:
                print("T) PASSED! (by raising an error as expected)")
                print(res.stderr)
                passed.append(test)
                #print("")
                continue
        else:
            output = res.stdout.rstrip().splitlines() # strip to remove newline
            #print("OUTPUT:")
            #for line in output:
            #    print("  {}".format(line))
            #print("<end>")
            if len(output) != len(test.cases):
                failed[test.type].append(TestFailure(test, []))
                print("TEST FAILED!")
                print("ERROR: Got {} lines of output, expected {}!".format(len(output), len(test.cases)))
                print_tests([test])
                print("RECEIVED OUTPUT:")
                for line in output:
                    print("-> {}".format(line))
                
                continue
        
        # Match each line with the expected output
        maxw = len(str(len(test.cases)))
        tag_template = "{{:<{}}}) ".format(maxw)
        output_line = 0
        for case_no, case in enumerate(test.cases, 1):
            tag = tag_template.format(case_no)
            cmd = run_cmd + case.input_lines
            inputstr = " <newline> ".join(case.input_lines) 
            
            if output_line >= len(output):
                print("{}ERROR!  : {}".format(tag, inputstr))
                print(res.stderr)
                failed_cases.append(case_no)
                break
            
            case_output = output[output_line]
            if case_output == case.expected_output:
                print("{}PASSED! : {} -> {}".format(tag, inputstr, case_output))
            else:
                print("{}FAILED! : {}".format(tag, inputstr))
                print("    Input:    {!r}\n".format(inputstr))
                print("    Expected: {!r}\n".format(case.expected_output))
                print("    Got:      {!r}\n".format(case_output))
                failed_cases.append(case_no)
            
            output_line += 1
        
        if failed_cases:
            failed[test.type].append(TestFailure(test, failed_cases))
        else:
            passed.append(test)
        
        print("")
    
    return (passed, failed)


def print_results(passed, failed):
    
    def print_failure(specifier, failed_tests):
        t = "test" if len(failed_tests) == 1 else "tests"
        print("{} {} {} failed".format(len(failed_tests), specifier, t))
        for failure in failed_tests:
            case_text = ", ".join(str(cn) for cn in failure.case_numbers)
            print("  - '{}' [ {} ]".format(failure.test.name, case_text))
        print("")
    
    verdict = "SUCCES" if not failed[TestType.Mandatory] else "FAILURE"
    if all(not tests for tests in failed.values()):
        verdict = "PERFECT"
    
    print("")
    print(" Test Results ".center(80, "="))
    t = "test" if len(passed) == 1 else "tests"
    print("{} {} passed\n".format(len(passed), t))
    print_failure("mandatory", failed[TestType.Mandatory])
    print_failure("deferrable", failed[TestType.Deferrable])
    print_failure("optional", failed[TestType.Optional])
    print("")
    print("Verdict: {}!".format(verdict))
    print("")
    
            

def load_tests(step_name):
    filepath = os.path.join(TEST_DIR, step_name+".mal")
    if not os.path.exists(filepath):
        raise FileNotFoundException("Could not find test file: {!r}".format(step_name+".mal"))
    with open(filepath, "r") as f:
        text = f.read()
    
    tests = []
    test_type = TestType.Mandatory
    test_should_fail = False
    
    test_name = "<Unnamed Test>"
    cases = []
    case_input_lines = []
    def start_new_test(lineno):
        if cases:
            if case_input_lines:
                raise Exception("Line {}: New test start, but no output for previous test case")
            test = Test(test_name, cases.copy(), test_type, test_should_fail)
            tests.append(test)
            cases.clear()
        
        
    for i, line in enumerate(text.splitlines()):
        line = line.strip()
        if line.startswith(";;"):
            start_new_test(i+1)
            test_name = line[2:].strip()
            test_should_fail = False
            continue
        
        elif line == "" or line.isspace():
            continue
        
        elif line.startswith(";>>>"):
            ll = line.lower()
            if "optional" in ll:
                test_type = TestType.Optional
            elif "deferrable" in ll:
                test_type = TestType.Deferrable
            else:
                raise Exception("Line {}: Unknown parse directive: {!r}", i+1, line)
        
        elif line.startswith(";=>"):
            #print("{}: OUTPUT: {}".format(i+1, line))
            if case_input_lines:
                output = line[3:].strip()
                case = TestCase(case_input_lines.copy(), output)
                cases.append(case)
                case_input_lines.clear()
            else:
                raise Exception("Line {}: Found output line with no input".format(i+1))
        
        elif line.startswith("; expected"):
            if not case_input_lines:
                raise Exception("Line {}: Found output line with no input".format(i+1))
            test_should_fail = True
            case = TestCase(case_input_lines.copy(), "")
            cases.append(case)
            case_input_lines.clear()
        
        elif line.startswith(";"):
            if "not found" in line:
                if not case_input_lines:
                    raise Exception("Line {}: Found output line with no input".format(i+1))
                test_should_fail = True
                case = TestCase(case_input_lines.copy(), "")
                cases.append(case)
                case_input_lines.clear()
            
            else:
                print("WARN: Line {}: Found nonspecial line starting with ';'".format(i+1))
                print("  {}".format(line))
            
            continue
        
        else:
            #print("{}: INPUT: {}".format(i+1, line))
            if not case_input_lines:
                case_input_lines.append(line)
            else:
                case_input_lines.append(line)
                #print("WARN: Line {}: Found second input line in a row".format(i+1))
    
    if case_input_lines:
        raise Exception("END: No output found for last input line")
    
    start_new_test(i+1)
    
    return tests


def print_tests(tests):
    print("Tests:")
    for test in tests:
        failtext = " <Should fail> " if test.should_fail else ""
        print("TEST: {} ({}){}".format(test.name, test.type, failtext))
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

            output = "<Error>" if test.should_fail else "-> " + case.expected_output
            print("{}| {}".format(itag, output))
            print("")
            #print("-"*(maxw+1))
        
    

def main(args=sys.argv[1:]):
    if not args:
        return print("Usage: python3 test.py <step>")
    from pprint import pprint
    step_name = get_step(args[0])
    tests = load_tests(step_name)
    #print_tests(tests)
    
    build_rust(step_name)
    cmd = rust_cmd(step_name)
    (passed, failed) = run_tests(tests, cmd)
    print_results(passed, failed)


if __name__ == '__main__':
    main()

