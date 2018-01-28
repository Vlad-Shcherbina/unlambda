import glob
import subprocess
import re
import sys

MAX_RUNS = 10
MIN_RUNS = 3
MAX_TIME = 5


def run(args, filename):
    p = subprocess.Popen(
        args + [filename],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE)

    out, err = p.communicate()
    returncode = p.wait()

    err = err.decode()
    if returncode:
        if 'does not support call/cc' in err:
            return False, '---'
        elif 'overflowed its stack' in err:
            return False, 'S/O'
        else:
            print('out:', out.decode())
            print('err:', repr(err))
            assert False

    with open(filename, 'rb') as fin:
        assert fin.read() == out, 'not a quine'

    m = re.match(r'It took ([\d\.]+)s', err)
    t = float(m.group(1))
    return True, t


def main():
    subprocess.check_call(['cargo', 'build', '--release'])

    interpreters = []
    modes = [
        'metacircular',
        'cps',
        'smallstep',
    ]
    for mode in modes:
        args = ['target/release/unlambda.exe',
                '--interpreter', mode,
                '--time']
        interpreters.append((mode, args))

    # To measure against https://github.com/bwo/unlambda/blob/master/unlambda.rs
    # interpreters.append(('bwo', ['bwo_unlambda.exe']))

    print(f'{"program":<45}', end='')
    for name, _ in interpreters:
        print(f'{name:>15}', end='')
    print()

    for filename in glob.glob("CUAN/quine/**/*.unl"):
        print(f'{filename:<45}', end='')
        for _, args in interpreters:
            sys.stdout.flush()
            ts = []
            for _ in range(MAX_RUNS):
                c, t = run(args, filename)
                ts.append(t)
                if not c:
                    break
                if sum(ts) > MAX_TIME and len(ts) >= MIN_RUNS:
                    break
            if not c:
                print(f'{ts[0]:>15}', end='')
            else:
                print(f'{min(ts):>15.5f}', end='')
        print()


if __name__ == '__main__':
    main()
