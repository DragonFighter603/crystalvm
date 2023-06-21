import sys
import re
import struct

from instructions_map import instr_to_bits

start_addr = 0x0008DE00

if len(sys.argv) < 3:
    print('usage: assepbler.py <in.casm> <out.cstl>')
    exit(1)
file = sys.argv[1]
outfile = sys.argv[2]

include_breakpoints = False
debug_info_file = None

args = sys.argv[3:]

for arg in args:
    if arg in ['--breakpoints', '-b']:
        include_breakpoints = True
    elif arg.startswith('--debug-info-file=') or arg.startswith('-d='):
        debug_info_file = arg.split('=', maxsplit=1)
    else:
        print(f'unhandled argument "{arg}"')

with open(file, 'r') as source_file:
    lines = source_file.readlines()

def error(err, line):
    print(f'Error in line {line+1}: {err}')
    exit(1)

class Here:
    def __init__(self):
        self.offset = 0
        self.resolved = None
        if current_section is None:
            raise Exception('Cannot use $ outside of a section')
        current_section.instructions.append(Location(self))

    def __add__(self, other):
        h = Here()
        h.offset = self.offset + other
        return h

    def __sub__(self, other):
        h = Here()
        h.offset = self.offset - other
        return h
    
    def __repr__(self) -> str:
        if self.resolved is not None:
            return f'{self.resolved + self.offset}'
        if self.offset == 0:
            return f'$0x{id(self):X}'
        elif self.offset > 0:
            return f'$0x{id(self):X} + {self.offset}'
        else:
            return f'$0x{id(self):X} - {abs(self.offset)}'

class uint:
    def __init__(self, value):
        self.value = value

    def __add__(self, other):
        value = self.value + other
        if value < 0:
            raise Exception(f'Uint cannot be negative: {value}!')
        return uint(value)

    def __sub__(self, other):
        value = self.value - other
        if value < 0:
            raise Exception(f'Uint cannot be negative: {value}!')
        return uint(value)
    
    def __mul__(self, other):
        value = self.value * other
        if value < 0:
            raise Exception(f'Uint cannot be negative: {value}!')
        return uint(value)
    
    def __floordiv__(self, other):
        value = self.value // other
        if value < 0:
            raise Exception(f'Uint cannot be negative: {value}!')
        return uint(value)
    
    def __repr__(self) -> str:
        return f'0x{self.value:X}u'
    
    def __str__(self) -> str:
        return f'uint(0x{self.value:X})'

class Section:
    def __init__(self, addr) -> None:
        self.addr = addr
        self.instructions = []

class Location:
    def __init__(self, here) -> None:
        self.here = here

    def __repr__(self) -> str:
        return f'Location({self.here})'

class Stack:
    def __repr__(self) -> str:
        return 'Stack'
    
class Variable:
    def __init__(self, name):
        self.name = name

    def __repr__(self) -> str:
        return f'Variable({self.name})'

class Literal:
    def __init__(self, ty, value):
        self.type = ty
        if self.type == 'byte':
            self.value = eval_var(value)
        elif self.type == 'word':
            self.value = eval_var(value)
        elif self.type == 'ascii':
            self.value = eval(value).encode("utf-8")
        else:
            raise Exception(f'Invalid type for literal: {self.type}')
    
    def __repr__(self) -> str:
        if self.type == 'ascii':
            return f'Literal({self.value}: {self.type})'
        else:
            return f'Literal({self.value}: {self.type})'
    
    def __len__(self) -> int:
        if self.type == 'byte':
            return 1
        if self.type == 'word':
            return 4
        if self.type == 'ascii':
            return len(self.value)
    
    def write_to(self, file):
        if self.type == 'byte':
            write_value(file, self.value, bytes_len=1)
        elif self.type == 'word':
            write_value(file, self.value)
        elif self.type == 'ascii':
            file.write(self.value)

class Instruction:
    def __init__(self, cmd, args):
        self.cmd = cmd
        self.args = args

    def __repr__(self) -> str:
        return f'{self.cmd} {" ".join([str(arg) for arg in self.args])}'

class Breakpoint:
    def __init__(self, tag):
        self.tag=tag

    def __repr__(self) -> str:
        return f'Breakpoint({self.tag})'

def eval_var(var: str, line=-1):
    var = var.strip()
    if var == '!':
        return Stack()
    if var[0] == '%':
        return var
    if var[0] == '@':
        if var[1:] in vars:
            return vars[var[1:]]
        return Variable(var[1:])
        # error(f'No such macro variable: "{var[1:]}"', line)
    if var[-1] == 'u':
        return uint(int(var[:-1], base=0))
    if len(var) > 2 and not var[1].isnumeric():
        return uint(int(var, base=0))
    return int(var, base=0)

vars = { 'Here': Here, 'uint': uint }

sections = []
current_section = Section(uint(start_addr))


def resolve(section):
    d = 0
    for c, instr in enumerate(section.instructions):
        if type(instr) == Location:
            instr.here.resolved = section.addr + c * 4 + d   
            d -= 4
        elif type(instr) == Instruction:
            d += sum([type(x) != str for x in instr.args]) * 4
        elif type(instr) == Literal:
            d += len(instr) - 4
        elif type(instr) == Breakpoint:
            pass
        else:
            raise Exception(f'invalid instruction "{instr}" of type {type(instr)}')

for i, line in enumerate(lines):
    line = line.strip()
    if len(line) == 0 or line.startswith(';'):
        continue
    if line[0] == ';':
        # comment
        pass
    if line[0] == '#':
        name, expr = line[1:].strip().split(' ', 1)
        expr = expr.replace('$', 'Here()')
        for match in re.findall(r'0[xX][0-9A-Fa-f]+|0[oO][0-7]+|0[bB][01]+', expr):
            expr = expr.replace(match, str(eval_var(match)))
        try:
            if name in vars and name != '_':
                error(f'variable {name} already exists, can\'t be redefined', i)
            vars[name] = eval(expr, vars)
        except Exception as e:
            error(e, i)
    elif line[0] == '~':
        resolve(current_section)
        addr = eval_var(line[1:].strip(), line=i)
        if type(addr) == Here:
            if addr.resolved is not None:
                addr = addr.resolved + addr.offset
        if type(addr) != uint:
            error(f'section has to be resolved to an uint: {addr}', i)
        if addr.value < start_addr:
            error(f'no sections before {start_addr:X} allowed: {addr}', i)
        section = Section(addr)
        sections.append(section)
        current_section = section
    elif line[0] == '.':
        ty, expr = line[1:].strip().split(' ', 1)
        current_section.instructions.append(Literal(ty, expr))
    elif line.startswith('breakpoint'):
        b, *tag = line.split(' ', 1)
        if include_breakpoints:
            tag = eval(tag[0], vars) if len(tag) > 0 else ''
            if len(tag) > 2:
                error('breakpoint tag is too long, has to be 2 or less chars: "{tag}"', i)
            current_section.instructions.append(Breakpoint(tag))
    else:
        args = line.split()
        cmd = args[0]
        cmd_args = []
        for a in args[1:]:
            a = a.strip()
            if len(a) > 0:
                r = eval_var(a, line=i)
                cmd_args.append(r)
        current_section.instructions.append(Instruction(cmd, cmd_args))

resolve(current_section)

sections = sorted(sections, key=lambda s: s.addr.value)


def write_value(file, num, bytes_len=4) -> bool:
    if type(num) == int:
        file.write(num.to_bytes(bytes_len, 'big'))
        return True
    elif type(num) == uint:
        file.write(num.value.to_bytes(bytes_len, 'big'))
        return True
    elif type(num) == Here:
        file.write((num.resolved + num.offset).value.to_bytes(bytes_len, 'big'))
        return True
    elif type(num) == float:
        file.write(struct.pack('f', num))
        return True
    return False

for section in sections:
    for instr in section.instructions:
        if type(instr) == Instruction:
            for i in range(0, len(instr.args)):
                if type(instr.args[i]) == Variable:
                    instr.args[i] = vars[instr.args[i].name]

with open(outfile, 'wb') as outbin:
    current_addr = start_addr
    for section in sections:
        print(f'{section.addr.value-current_addr} buffer bytes')
        if section.addr.value < current_addr:
            raise Exception(f'Invalid location {section.addr.value:08X}, already are at {current_addr:08X}')
        outbin.write(bytes(section.addr.value-current_addr))
        current_addr = section.addr.value
        for instr in section.instructions:
            print(instr)
            if type(instr) == Variable:
                instr = vars[instr.name]

            if type(instr) == Location:
                # marker
                pass
            elif type(instr) == Literal:
                instr.write_to(outbin)
                current_addr += len(instr)
            elif type(instr) == Breakpoint:
                outbin.write(bytes(int('1111111111100000', base=2).to_bytes(4, 'big')))
                outbin.write(instr.tag.ljust(2).encode('utf-8'))
                current_addr += 4
            elif type(instr) == Instruction:
                cmd = instr.cmd
                b = instr_to_bits(cmd)
                nums = []
                for arg in instr.args:
                    if type(arg) == str:
                        if arg[0] == '%' and arg[1:].isnumeric():
                            n = int(arg[1:], base=16)
                            if n > 48:
                                raise Exception(f'invalid trg num "{arg}"')
                            b += '0' + f'{n:06b}'
                        elif len(arg) == 2 and arg[0] == '%' and arg[1] in ['S', 'I', 'L', 'C', 'F', 'Q']:
                            n = ['S', 'I', 'L', 'C', 'F', 'Q'].index(arg[1]) + 48
                            b += '0' + f'{n:06b}'
                        else:
                            raise Exception(f'invalid arg "{arg}"')
                    elif type(arg) == Stack:
                        b += '1000000'
                    else:
                        b += '1111111'
                        nums.append(arg)
                b += '0' * (32-len(b))
                outbin.write(bytes(int(b, base=2).to_bytes(4, 'big')))
                current_addr += 4
                for num in nums:
                    current_addr += 4
                    if not write_value(outbin, num):
                        raise Exception(f'invalid argument "{num}" of type {type(instr)} for {" ".join([str(i) for i in instr])}')
            else:
                raise Exception(f'invalid instruction "{instr}" of type {type(instr)}')