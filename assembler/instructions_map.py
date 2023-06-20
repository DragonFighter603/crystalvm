def instr_to_bits(instr: str):
    match instr:
        case 'noop':   return '00000000000'
        case 'addu':   return '00000000001'
        case 'subu':   return '00000000010'
        case 'mulu':   return '00000000011'
        case 'divu':   return '00000000100'
        case 'modu':   return '00000000101'
        case 'cmpu':   return '00000000111'
        case 'addi':   return '00000010001'
        case 'subi':   return '00000010010'
        case 'muli':   return '00000010011'
        case 'divi':   return '00000010100'
        case 'modi':   return '00000010101'
        case 'cmpi':   return '00000010111'
        case 'absi':   return '00000011000'
        case 'powi':   return '00000011001'
        case 'and':    return '00000001000'
        case 'or':     return '00000001001'
        case 'xor':    return '00000001010'
        case 'not':    return '00000001011'
        case 'shl':    return '00000001100'
        case 'shr':    return '00000001101'
        case 'rol':    return '00000001110'
        case 'ror':    return '00000001111'
        case 'itu':    return '00000011100'
        case 'uti':    return '00000011101'
        case 'itf':    return '00000011110'
        case 'fti':    return '00000011111'
        case 'addf':   return '00000100000'
        case 'subf':   return '00000100001'
        case 'mulf':   return '00000100010'
        case 'divf':   return '00000100011'
        case 'modf':   return '00000100100'
        case 'absf':   return '00000100101'
        case 'powfi':  return '00000100110'
        case 'powff':  return '00000110110'
        case 'cmpf':   return '00000100111'
        case 'sqrt':   return '00000101000'
        case 'exp':    return '00000101001'
        case 'log':    return '00000101010'
        case 'ln':     return '00000111010'
        case 'sin':    return '00000101011'
        case 'asin':   return '00000101100'
        case 'cos':    return '00000101101'
        case 'tan':    return '00000101110'
        case 'atan':   return '00000101111'
        case 'sinh':   return '00000110000'
        case 'asih':   return '00000110001'
        case 'cosh':   return '00000110010'
        case 'acoh':   return '00000110011'
        case 'jmp':    return '00001000000'
        case 'jz':     return '00001000010'
        case 'jnz':    return '00001000011'
        case 'jl':     return '00001000100'
        case 'jnl':    return '00001000101'
        case 'jc':     return '00001000110'
        case 'jnc':    return '00001000111'
        case 'jo':     return '00001001000'
        case 'jno':    return '00001001001'
        case 'call':   return '00001010000'
        case 'ret':    return '00001010001'
        case 'move':   return '00010000000'
        case 'ld':     return '00010000001'
        case 'st':     return '00010000011'
        case 'ldb':    return '00010000101'
        case 'stb':    return '00010000111'
        case 'dup':    return '00010001000'
        case 'over':   return '00010001001'
        case 'srl':    return '00010001010'
        case 'srr':    return '00010001011'
        case 'enter':  return '00010001100'
        case 'leave':  return '00010001101'
        case 'pshar':  return '00010001110'
        case 'resar':  return '00010001111'
        case 'time':   return '00011100000'
        case 'wait':   return '00011100001'
        case 'dread':  return '00011101001'
        case 'dwrite': return '00011101010'
        case 'dquery': return '00011101011'
        case x: raise Exception(f'invalid instruction "{x}"')

def bits_to_instr(bits: str):
    match bits:
        case '00000000000': return 'noop'
        case '00000000001': return 'addu'
        case '00000000010': return 'subu'
        case '00000000011': return 'mulu'
        case '00000000100': return 'divu'
        case '00000000101': return 'modu'
        case '00000000111': return 'cmpu'
        case '00000010001': return 'addi'
        case '00000010010': return 'subi'
        case '00000010011': return 'muli'
        case '00000010100': return 'divi'
        case '00000010101': return 'modi'
        case '00000010111': return 'cmpi'
        case '00000011000': return 'absi'
        case '00000011001': return 'powi'
        case '00000001000': return 'and'
        case '00000001001': return 'or'
        case '00000001010': return 'xor'
        case '00000001011': return 'not'
        case '00000001100': return 'shl'
        case '00000001101': return 'shr'
        case '00000001110': return 'rol'
        case '00000001111': return 'ror'
        case '00000011100': return 'itu'
        case '00000011101': return 'uti'
        case '00000011110': return 'itf'
        case '00000011111': return 'fti'
        case '00000100000': return 'addf'
        case '00000100001': return 'subf'
        case '00000100010': return 'mulf'
        case '00000100011': return 'divf'
        case '00000100100': return 'modf'
        case '00000100101': return 'absf'
        case '00000100110': return 'powfi'
        case '00000110110': return 'powff'
        case '00000100111': return 'cmpf'
        case '00000101000': return 'sqrt'
        case '00000101001': return 'exp'
        case '00000101010': return 'log'
        case '00000111010': return 'ln'
        case '00000101011': return 'sin'
        case '00000101100': return 'asin'
        case '00000101101': return 'cos'
        case '00000101110': return 'tan'
        case '00000101111': return 'atan'
        case '00000110000': return 'sinh'
        case '00000110001': return 'asih'
        case '00000110010': return 'cosh'
        case '00000110011': return 'acoh'
        case '00001000000': return 'jmp'
        case '00001000010': return 'jz'
        case '00001000011': return 'jnz'
        case '00001000100': return 'jl'
        case '00001000101': return 'jnl'
        case '00001000110': return 'jc'
        case '00001000111': return 'jnc'
        case '00001001000': return 'jo'
        case '00001001001': return 'jno'
        case '00001010000': return 'call'
        case '00001010001': return 'ret'
        case '00010000000': return 'move'
        case '00010000001': return 'ld'
        case '00010000011': return 'st'
        case '00010000101': return 'ldb'
        case '00010000111': return 'stb'
        case '00010001000': return 'dup'
        case '00010001001': return 'over'
        case '00010001010': return 'srl'
        case '00010001011': return 'srr'
        case '00010001100': return 'enter'
        case '00010001101': return 'leave'
        case '00010001110': return 'pshar'
        case '00010001111': return 'resar'
        case '00011100000': return 'time'
        case '00011100001': return 'wait'
        case '00011101001': return 'dread' 
        case '00011101010': return 'dwrite' 
        case '00011101011': return 'dquery' 
        case x: raise Exception(f'invalid instruction "{x}"')
