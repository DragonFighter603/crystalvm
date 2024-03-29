macro_rules! define_var {
    ($instr: ident, $instr_str: ident, $instr_str_val: ident, $doc: literal, $index: expr) => {
        #[doc = $doc]
        #[allow(unused)]
        pub const $instr: u32 = $index;
        #[doc = $doc]
        #[allow(unused)]
        pub const $instr_str: &str = stringify!($instr_str_val);
    };
}

macro_rules! define_vars {
    ($i: ident, $is: ident, $isv: ident, $doc: literal, $len: literal) => {
        define_var!($i, $is, $isv, $doc, $len-1);
    };
    ($i: ident, $is: ident, $isv: ident, $d: literal, $( $instr: ident, $instr_str: ident, $instr_str_val: ident, $doc: literal, )* $len: literal) => {
        define_var!($i, $is, $isv, $d, $len-${count(instr, 0)}-1);
        define_vars!($( $instr, $instr_str, $instr_str_val, $doc, )* $len);
    }
}

macro_rules! define_instructions {
    (regs $a: ident, $b: ident, $c: ident; context $self: ident; $(instr $instr: ident { $instr_str: ident = $instr_str_val: ident; $action: expr; $doc: literal; })*) => {
        define_vars!($( $instr, $instr_str, $instr_str_val, $doc, )* ${count(instr, 0)});
        macro_rules! impl_instructions_match {
            ($pass_self: ident, $ins: ident, $pass_a: ident, $pass_b: ident, $pass_c: ident) => { {
                let $a = $pass_a;
                let $b = $pass_b;
                let $c = $pass_c;
                let $self = $pass_self;
                match $ins {
                    $( $instr => $action, )*
                    _ => unsafe { $self.mutator().registers[REG_F as usize] |= FLAG_BIT_E; }
                }
            } }
        }
        pub(crate) use impl_instructions_match; 
        #[allow(unused)]
        pub fn instr_name_id_map() -> std::collections::HashMap<&'static str, u32>{
            let mut map = std::collections::HashMap::default();
            $( map.insert($instr_str, $instr); )*
            map
        }
        #[allow(unused)]
        pub fn instr_id_name_map() -> std::collections::HashMap<u32, &'static str>{
            let mut map = std::collections::HashMap::default();
            $( map.insert($instr, $instr_str,); )*
            map
        }
    };
}

macro_rules! carry_handler {
    (bool $x: expr => in) => { $x & 1 != 0 };
    (bool $x: expr => out) => { $x as u32 };
    ($ty: ident $x: expr => out) => { unsafe { std::mem::transmute::<$ty, u32>($x) } };
    ($ty: ident $x: expr => in) => { unsafe { std::mem::transmute::<u32, $ty>($x) } };
}

macro_rules! impl_arith_ret {
    ($self: ident, $ret: ident, $ret_ty: ident, write to reg $reg: ident) => {
        $self.write_arg($reg, unsafe { std::mem::transmute::<$ret_ty, u32>($ret) });
    };
    ($self: ident, $ret: ident, bool, overflow) => {
        unsafe {
            let mutor = $self.mutator();
            mutor.registers[REG_F as usize] |= ($ret as u32) << FLAG_PLACE_C;
        }
    };
    ($self: ident, $ret: ident, $ret_ty: ident, carry) => {
        unsafe {
            let mutor = $self.mutator();
            mutor.registers[REG_C as usize] = carry_handler!($ret_ty $ret => out);
            mutor.registers[REG_F as usize] |= ((mutor.registers[REG_C as usize] != 0) as u32) << FLAG_PLACE_C;
        }
    };
    ($self: ident, $ret: ident, $ret_ty: ident, write to reg $reg: ident as $inner_ty: ident and on error $err_flag: ident) => {
        match $ret{
            Some(res) => $self.write_arg($reg, unsafe { std::mem::transmute::<$inner_ty, u32>(res) }),
            None => unsafe { $self.mutator().registers[REG_F as usize] |= $err_flag }
        }
    }
}


macro_rules! some_or_default {
    (, $dflt: tt) => {
        $dflt
    };
    ($t: tt, $dflt: tt) => {
        $t
    };
}

macro_rules! impl_func {
    ($self: ident || $expr: expr => ($($ret: ident: $ret_ty: ident => [$($ret_handle: ident)*]),*)) => { {
        let ( $( $ret ),* ) = $expr;
        $( impl_arith_ret!($self, $ret, $ret_ty, $($ret_handle)*); )*
    } };
    ($self: ident |$($arg:ident: $arg_ty: ident $(as $alias: ident)?),* $(+ $carry: ident: $carry_ty: ident)?| $expr: expr => ($($ret: ident: $ret_ty: ident => [$($ret_handle: ident)*]),*)) => { {
        $( let some_or_default!($($alias)?, $arg) = unsafe { std::mem::transmute::<u32, $arg_ty>($self.read_arg($arg)) }; )*
        $( let $carry = carry_handler!($carry_ty $self.registers[REG_C as usize] => in); )?
        let ( $( $ret ),* ) = $expr;
        $( impl_arith_ret!($self, $ret, $ret_ty, $($ret_handle)*); )*
    } }
}

macro_rules! impl_jump {
    ($self: ident jump $a: ident) => { {
        let a = $self.read_arg($a);
        unsafe {  $self.mutator().registers[REG_I as usize] = a; }
    } };
    ($self: ident jump $a: ident if $mask: ident) => { {
        let a = $self.read_arg($a);
        if $self.registers[REG_F as usize] & $mask != 0 {
            unsafe {  $self.mutator().registers[REG_I as usize] = a; }
        }
    } };
    ($self: ident jump $a: ident unless $mask: ident) => { {
        let a = $self.read_arg($a);
        if $self.registers[REG_F as usize] & $mask == 0 {
            unsafe {  $self.mutator().registers[REG_I as usize] = a; }
        }
    } };
    ($self: ident clear $mask: ident) => { {
        unsafe {  $self.mutator().registers[REG_F as usize] &= !$mask; }
    } };
}

pub(crate) use impl_func; 
pub(crate) use impl_arith_ret; 
pub(crate) use carry_handler; 
pub(crate) use impl_jump; 
pub(crate) use some_or_default;

pub(crate) type MaybeU32 = Option<u32>;
pub(crate) type MaybeI32 = Option<i32>;
pub(crate) type MaybeF32 = Option<f32>;

define_instructions! {
    regs a, b, c;
    context thread;
    instr INSTR_NOOP { INSTR_NOOP_STR = noop; (); "no-op instruction"; }
    instr INSTR_ADD { INSTR_ADD_STR = add; impl_func!(thread |a: u32, b: u32| u32::overflowing_add(a, b) => (r: u32 => [write to reg c], o: bool => [carry])); "u32: a + b = c + carry"; }
    instr INSTR_SUB { INSTR_SUB_STR = sub; impl_func!(thread |a: u32, b: u32| u32::overflowing_sub(a, b) => (r: u32 => [write to reg c], o: bool => [carry])); "u32: a - b = c + carry"; }
    instr INSTR_MUL { INSTR_MUL_STR = mul; impl_func!(thread |a: u32, b: u32| u32::widening_mul(a, b) => (r: u32 => [write to reg c], o: u32 => [carry])); "u32: a * b = c + carry"; }
    instr INSTR_DIV { INSTR_DIV_STR = div; impl_func!(thread |a: u32, b: u32| u32::checked_div(a, b) => (r: MaybeU32 => [write to reg c as u32 and on error FLAG_BIT_L])); "u32: a / b = c, FLAG_BIT_L if b == 0"; }
    instr INSTR_REM { INSTR_REM_STR = rem; impl_func!(thread |a: u32, b: u32| u32::checked_rem(a, b) => (r: MaybeU32 => [write to reg c as u32 and on error FLAG_BIT_L])); "u32: a % b = c, FLAG_BIT_L if b == 0"; }
    instr INSTR_POW { INSTR_POW_STR = pow; impl_func!(thread |a: u32, b: u32| u32::checked_pow(a, b) => (r: MaybeU32 => [write to reg c as u32 and on error FLAG_BIT_C])); "u32: a ** b = c, FLAG_BIT_C if overflow"; }
    instr INSTR_MIN { INSTR_MIN_STR = min; impl_func!(thread |a: u32, b: u32| u32::min(a, b) => (r: u32 => [write to reg c])); "u32: min(a, b) = c"; }
    instr INSTR_MAX { INSTR_MAX_STR = max; impl_func!(thread |a: u32, b: u32| u32::max(a, b) => (r: u32 => [write to reg c])); "u32: max(a, b) = c"; }
    
    instr INSTR_CADD { INSTR_CADD_STR = cadd; impl_func!(thread |a: u32, b: u32 + carry: bool| u32::carrying_add(a, b, carry) => (r: u32 => [write to reg c], o: bool => [carry])); "u32: a + b + carry = c + carry"; }
    instr INSTR_CSUB { INSTR_CSUB_STR = csub; impl_func!(thread |a: u32, b: u32 + carry: bool| u32::borrowing_sub(a, b, carry) => (r: u32 => [write to reg c], o: bool => [carry])); "u32: a - b - carry = c + carry"; }
    instr INSTR_CMUL { INSTR_CMUL_STR = cmul; impl_func!(thread |a: u32, b: u32 + carry: u32| u32::carrying_mul(a, b, carry) => (r: u32 => [write to reg c], o: u32 => [carry])); "u32: a * b + carry = c + carry"; }
    
    instr INSTR_IADD { INSTR_IADD_STR = iadd; impl_func!(thread |a: i32, b: i32| i32::overflowing_add(a, b) => (r: i32 => [write to reg c], o: bool => [carry])); "i32: a + b = c + carry"; }
    instr INSTR_ISUB { INSTR_ISUB_STR = isub; impl_func!(thread |a: i32, b: i32| i32::overflowing_sub(a, b) => (r: i32 => [write to reg c], o: bool => [carry])); "i32: a - b = c + carry"; }
    instr INSTR_IMUL { INSTR_IMUL_STR = imul; impl_func!(thread |a: i32, b: i32| i32::overflowing_mul(a, b) => (r: i32 => [write to reg c], o: bool => [overflow])); "i32: a * b = c, FLAG_BIT_C if overflow"; }
    instr INSTR_IDIV { INSTR_IDIV_STR = idiv; impl_func!(thread |a: i32, b: i32| i32::checked_div(a, b) => (r: MaybeI32 => [write to reg c as i32 and on error FLAG_BIT_L])); "i32: a / b = c, FLAG_BIT_L if b == 0"; }
    instr INSTR_IREM { INSTR_IREM_STR = irem; impl_func!(thread |a: i32, b: i32| i32::checked_rem(a, b) => (r: MaybeI32 => [write to reg c as i32 and on error FLAG_BIT_L])); "i32: a % b = c, FLAG_BIT_L if b == 0"; }
    instr INSTR_IREME { INSTR_IREME_STR = ireme; impl_func!(thread |a: i32, b: i32| i32::checked_rem_euclid(a, b) => (r: MaybeI32 => [write to reg c as i32 and on error FLAG_BIT_L])); "i32: a % b = c (euclidian), FLAG_BIT_L if b == 0"; }
    instr INSTR_IABS { INSTR_IABS_STR = iabs; impl_func!(thread |a: i32| i32::abs(a) => (r: i32 => [write to reg b])); "i32: |a|"; }
    instr INSTR_IPOW { INSTR_IPOW_STR = ipow; impl_func!(thread |a: i32, b: u32| i32::overflowing_pow(a, b) => (r: i32 => [write to reg c], o: bool => [overflow])); "i32: a ** (u32)b = c, FLAG_BIT_C if overflow"; }
    instr INSTR_IMIN { INSTR_IMIN_STR = imin; impl_func!(thread |a: u32, b: u32| u32::min(a, b) => (r: u32 => [write to reg c])); "i32: min(a, b) = c"; }
    instr INSTR_IMAX { INSTR_IMAX_STR = imax; impl_func!(thread |a: u32, b: u32| u32::max(a, b) => (r: u32 => [write to reg c])); "i32: max(a, b) = c"; }
    
    instr INSTR_ICADD { INSTR_ICADD_STR = icadd; impl_func!(thread |a: i32, b: i32 + carry: bool| i32::carrying_add(a, b, carry) => (r: i32 => [write to reg c], o: bool => [carry])); "i32: a + b + carry = c + carry"; }
    instr INSTR_ICSUB { INSTR_ICSUB_STR = icsub; impl_func!(thread |a: i32, b: i32 + carry: bool| i32::borrowing_sub(a, b, carry) => (r: i32 => [write to reg c], o: bool => [carry])); "i32: a - b - carry = c + carry"; }

    instr INSTR_SHL { INSTR_SHL_STR = shl; impl_func!(thread |a: u32, b: u32| u32::overflowing_shl(a, b) => (r: u32 => [write to reg c], o: bool => [overflow])); "a << b = c, FLAG_BIT_C if overflow"; }
    instr INSTR_SHR { INSTR_SHR_STR = shr; impl_func!(thread |a: u32, b: u32| u32::overflowing_shr(a, b) => (r: u32 => [write to reg c], o: bool => [overflow])); "a >> b = c, FLAG_BIT_C if overflow"; }
    instr INSTR_WSHL { INSTR_WSHL_STR = wshl; impl_func!(thread |a: u32, b: u32| u32::wrapping_shl(a, b) => (r: u32 => [write to reg c])); "a << b = c (wrapping)"; }
    instr INSTR_WSHR { INSTR_WSHR_STR = wshr; impl_func!(thread |a: u32, b: u32| u32::wrapping_shr(a, b) => (r: u32 => [write to reg c])); "a >> b = c (wrapping)"; }
    instr INSTR_AND { INSTR_AND_STR = and; impl_func!(thread |a: u32, b: u32| a & b => (r: u32 => [write to reg c])); "a & b"; }
    instr INSTR_OR { INSTR_OR_STR = or; impl_func!(thread |a: u32, b: u32| a | b => (r: u32 => [write to reg c])); "a | b"; }
    instr INSTR_XOR { INSTR_XOR_STR = xor; impl_func!(thread |a: u32, b: u32| u32::wrapping_shl(a, b) => (r: u32 => [write to reg c])); "a ^ b = c"; }
    instr INSTR_NEG { INSTR_NEG_STR = neg; impl_func!(thread |a: u32| !a => (r: u32 => [write to reg b])); "!a = b"; }

    instr INSTR_CONVI2U { INSTR_CONVI2U_STR = convi2u; impl_func!(thread |a: i32| a as u32 => (r: u32 => [write to reg b])); "i32: a as u32 = b"; }
    instr INSTR_CONVU2I { INSTR_CONVU2I_STR = convu2i; impl_func!(thread |a: u32| a as i32 => (r: i32 => [write to reg b])); "u32: a as i32 = b"; }
    instr INSTR_CONVF2U { INSTR_CONVF2U_STR = convf2u; impl_func!(thread |a: f32| a as u32 => (r: u32 => [write to reg b])); "f32: a as u32 = b"; }
    instr INSTR_CONVU2F { INSTR_CONVU2F_STR = convu2f; impl_func!(thread |a: u32| a as f32 => (r: f32 => [write to reg b])); "u32: a as f32 = b"; }
    instr INSTR_CONVF2I { INSTR_CONVF2I_STR = convf2i; impl_func!(thread |a: f32| a as i32 => (r: i32 => [write to reg b])); "f32: a as i32 = b"; }
    instr INSTR_CONVI2F { INSTR_CONVI2F_STR = convi2f; impl_func!(thread |a: i32| a as f32 => (r: f32 => [write to reg b])); "i32: a as f32 = b"; }

    instr INSTR_FADD { INSTR_FADD_STR = fadd; impl_func!(thread |a: f32, b: f32| a + b => (r: f32 => [write to reg c])); "f32: a + b = c"; }
    instr INSTR_FSUB { INSTR_FSUB_STR = fsub; impl_func!(thread |a: f32, b: f32| a - b => (r: f32 => [write to reg c])); "f32: a - b = c"; }
    instr INSTR_FMUL { INSTR_FMUL_STR = fmul; impl_func!(thread |a: f32, b: f32| a * b => (r: f32 => [write to reg c])); "f32: a * b = c"; }
    instr INSTR_FDIV { INSTR_FDIV_STR = fdiv; impl_func!(thread |a: f32, b: f32| a / b => (r: f32 => [write to reg c])); "f32: a / b = c"; }
    instr INSTR_FREM { INSTR_FREM_STR = frem; impl_func!(thread |a: f32, b: f32| a % b => (r: f32 => [write to reg c])); "f32: a % b = c"; }
    instr INSTR_FREME { INSTR_FREME_STR = freme; impl_func!(thread |a: f32, b: f32| f32::rem_euclid(a, b) => (r: f32 => [write to reg c])); "f32: a % b = c (euclid)"; }
    instr INSTR_FABS { INSTR_FABS_STR = fabs; impl_func!(thread |a: f32, b: f32| f32::rem_euclid(a, b) => (r: f32 => [write to reg c])); "f32: a % b = c (euclid)"; }
    instr INSTR_FPOWI { INSTR_FPOWI_STR = fpowi; impl_func!(thread |a: f32, b: i32| f32::powi(a, b) => (r: f32 => [write to reg c])); "f32: a ** (i32)b = c"; }
    instr INSTR_FPOW { INSTR_FPOW_STR = fpow; impl_func!(thread |a: f32, b: f32| f32::powf(a, b) => (r: f32 => [write to reg c])); "f32: a ** b = c"; }
    instr INSTR_FLOOR { INSTR_FLOOR_STR = floor; impl_func!(thread |a: f32| f32::floor(a) => (r: f32 => [write to reg b])); "f32: floor(a) = b"; }
    instr INSTR_CEIL { INSTR_CEIL_STR = ceil; impl_func!(thread |a: f32| f32::ceil(a) => (r: f32 => [write to reg b])); "f32: ceil(a) = b"; }
    instr INSTR_ROUND { INSTR_ROUND_STR = round; impl_func!(thread |a: f32| f32::round(a) => (r: f32 => [write to reg b])); "f32: round(a) = b"; }
    instr INSTR_SIGN { INSTR_SIGN_STR = sign; impl_func!(thread |a: f32| f32::signum(a) => (r: f32 => [write to reg b])); "f32: sign(a) = b, 1 or -1 depending on sign"; }
    instr INSTR_FPART { INSTR_FPART_STR = fpart; impl_func!(thread |a: f32| f32::fract(a) => (r: f32 => [write to reg b])); "f32: fpart(a) = b, fractional part of floating point"; }
    instr INSTR_IPART { INSTR_IPART_STR = ipart; impl_func!(thread |a: f32| f32::trunc(a) => (r: f32 => [write to reg b])); "f32: ipart(a) = b, integer part of floating point"; }
    instr INSTR_RECIP { INSTR_RECIP_STR = recip; impl_func!(thread |a: f32| f32::recip(a) => (r: f32 => [write to reg b])); "f32: 1/x = b"; }
    instr INSTR_SQRT { INSTR_SQRT_STR = sqrt; impl_func!(thread |a: f32| f32::sqrt(a) => (r: f32 => [write to reg b])); "f32: sqrt(a) = b (b ** 2 = a)"; }
    instr INSTR_CBRT { INSTR_CBRT_STR = cbrt; impl_func!(thread |a: f32| f32::cbrt(a) => (r: f32 => [write to reg b])); "f32: qbrt(a) = b (b ** 3 = a)"; }
    instr INSTR_EXP { INSTR_EXP_STR = exp; impl_func!(thread |a: f32| f32::exp(a) => (r: f32 => [write to reg b])); "f32: e^a = b"; }
    instr INSTR_EXP2 { INSTR_EXP2_STR = exp2; impl_func!(thread |a: f32| f32::exp2(a) => (r: f32 => [write to reg b])); "f32: 2^a = b"; }
    instr INSTR_EXPM1 { INSTR_EXPM1_STR = expm1; impl_func!(thread |a: f32| f32::exp_m1(a) => (r: f32 => [write to reg b])); "f32: e^a - 1 = b"; }
    instr INSTR_LN { INSTR_LN_STR = ln; impl_func!(thread |a: f32| f32::ln(a) => (r: f32 => [write to reg b])); "f32: ln(a) = b"; }
    instr INSTR_LOG { INSTR_LOG_STR = log; impl_func!(thread |a: f32, b: f32| f32::log(a, b) => (r: f32 => [write to reg c])); "f32: log(a, b) = b"; }
    instr INSTR_LOG2 { INSTR_LOG2_STR = log2; impl_func!(thread |a: f32| f32::log2(a) => (r: f32 => [write to reg c])); "f32: ln2(a) = ln(a, 2) = c"; }
    instr INSTR_LOG10 { INSTR_LOG10_STR = log10; impl_func!(thread |a: f32| f32::log10(a) => (r: f32 => [write to reg b])); "f32: log10(a) = log(a, 10) = b"; }
    instr INSTR_LN1P { INSTR_LN1P_STR = ln1p; impl_func!(thread |a: f32| f32::ln_1p(a) => (r: f32 => [write to reg b])); "f32: ln(a + 1) = b"; }
    instr INSTR_FMIN { INSTR_FMIN_STR = fmin; impl_func!(thread |a: f32, b: f32| f32::min(a, b) => (r: f32 => [write to reg c])); "f32: min(a, b) = c"; }
    instr INSTR_FMAX { INSTR_FMAX_STR = fmax; impl_func!(thread |a: f32, b: f32| f32::max(a, b) => (r: f32 => [write to reg c])); "f32: max(a, b) = c"; }

    instr INSTR_SIN { INSTR_SIN_STR = sin; impl_func!(thread |a: f32| f32::sin(a) => (r: f32 => [write to reg b])); "f32: sin(a) = b"; }
    instr INSTR_ASIN { INSTR_ASIN_STR = asin; impl_func!(thread |a: f32| f32::asin(a) => (r: f32 => [write to reg b])); "f32: asin(a) = b"; }
    instr INSTR_SINH { INSTR_SINH_STR = sinh; impl_func!(thread |a: f32| f32::sinh(a) => (r: f32 => [write to reg b])); "f32: sinh(a) = b"; }
    instr INSTR_ASINH { INSTR_ASINH_STR = asinh; impl_func!(thread |a: f32| f32::asinh(a) => (r: f32 => [write to reg b])); "f32: asinh(a) = b"; }
    instr INSTR_COS { INSTR_COS_STR = cos; impl_func!(thread |a: f32| f32::cos(a) => (r: f32 => [write to reg b])); "f32: cos(a) = b"; }
    instr INSTR_ACOS { INSTR_ACOS_STR = acos; impl_func!(thread |a: f32| f32::acos(a) => (r: f32 => [write to reg b])); "f32: acos(a) = b"; }
    instr INSTR_COSH { INSTR_COSH_STR = cosh; impl_func!(thread |a: f32| f32::cosh(a) => (r: f32 => [write to reg b])); "f32: cosh(a) = b"; }
    instr INSTR_ACOSH { INSTR_ACOSH_STR = acosh; impl_func!(thread |a: f32| f32::acosh(a) => (r: f32 => [write to reg b])); "f32: acosh(a) = b"; }
    instr INSTR_TAN { INSTR_TAN_STR = tan; impl_func!(thread |a: f32| f32::tan(a) => (r: f32 => [write to reg b])); "f32: tan(a) = b"; }
    instr INSTR_ATAN { INSTR_ATAN_STR = atan; impl_func!(thread |a: f32| f32::atan(a) => (r: f32 => [write to reg b])); "f32: atan(a) = b"; }
    instr INSTR_TANH { INSTR_TANH_STR = tanh; impl_func!(thread |a: f32| f32::tanh(a) => (r: f32 => [write to reg b])); "f32: tanh(a) = b"; }
    instr INSTR_ATANH { INSTR_ATANH_STR = atanh; impl_func!(thread |a: f32| f32::atanh(a) => (r: f32 => [write to reg b])); "f32: atanh(a) = b"; }
    instr INSTR_ATAN2 { INSTR_ATAN2_STR = atan2; impl_func!(thread |a: f32, b: f32| f32::atan2(a, b) => (r: f32 => [write to reg c])); "f32: atan2(a, b) = c, where a is y and b is x"; }
    instr INSTR_SINCOS { INSTR_SINCOS_STR = sincos; impl_func!(thread |a: f32| f32::sin_cos(a) => (y: f32 => [write to reg b], x: f32 => [write to reg c])); "f32: sincos(a) = (sin(a), cos(a)) = (b, c)"; }
    instr INSTR_MAG2D { INSTR_MAG2D_STR = mag2d; impl_func!(thread |a: f32, b: f32| f32::max(a, b) => (r: f32 => [write to reg c])); "f32: sqrt(a*a+b*b) = c"; }

    instr INSTR_CMP { INSTR_CMP_STR = cmp; impl_func!(thread |a: u32, b: u32|{
        let mutor = unsafe { thread.mutator() };
        if a == b { mutor.registers[REG_F as usize] |= FLAG_BIT_Z; }
        else { mutor.registers[REG_F as usize] &= !FLAG_BIT_Z; }
        if a < b { mutor.registers[REG_F as usize] |= FLAG_BIT_S; }
        else { mutor.registers[REG_F as usize] &= !FLAG_BIT_S; }
    } => ()); "u32: cmp(a, b), FLAG_BIT_Z if a == b, FLAG_BIT_S if a < b "; }
    instr INSTR_ICMP { INSTR_ICMP_STR = icmp; impl_func!(thread |a: i32, b: i32|{
        let mutor = unsafe { thread.mutator() };
        if a == b { mutor.registers[REG_F as usize] |= FLAG_BIT_Z; }
        else { mutor.registers[REG_F as usize] &= !FLAG_BIT_Z; }
        if a < b { mutor.registers[REG_F as usize] |= FLAG_BIT_S; }
        else { mutor.registers[REG_F as usize] &= !FLAG_BIT_S; }
    } => ()); "i32: cmp(a, b), FLAG_BIT_Z if a == b, FLAG_BIT_S if a < b "; }
    instr INSTR_FCMP { INSTR_FCMP_STR = fcmp; impl_func!(thread |a: f32, b: f32|{
        let mutor = unsafe { thread.mutator() };
        if a == b { mutor.registers[REG_F as usize] |= FLAG_BIT_Z; }
        else { mutor.registers[REG_F as usize] &= !FLAG_BIT_Z; }
        if a < b { mutor.registers[REG_F as usize] |= FLAG_BIT_S; }
        else { mutor.registers[REG_F as usize] &= !FLAG_BIT_S; }
    } => ()); "f32: cmp(a, b), FLAG_BIT_Z if a == b, FLAG_BIT_S if a < b "; }

    instr INSTR_JMP { INSTR_JMP_STR = jmp; impl_jump!(thread jump a); "jump to addr"; }
    instr INSTR_CALL { INSTR_CALL_STR = call; unsafe {
        let mutor = thread.mutator();
        //println!("{:?}", mutor.registers);
        let addr = mutor.read_arg(a);
        mutor.registers[REG_S as usize] += 4;
        let base = mutor.registers[REG_S as usize];
        mutor.write_u32(mutor.registers[REG_S as usize], mutor.registers[REG_I as usize]);
        mutor.registers[REG_S as usize] += 4;
        mutor.write_u32(mutor.registers[REG_S as usize], mutor.registers[REG_B as usize]);
        mutor.registers[REG_B as usize] = base;
        mutor.registers[REG_I as usize] = addr;
        //println!("c {}", mutor.read_u32(mutor.registers[REG_S as usize - 12]));
        //println!("c {}", mutor.read_u32(mutor.registers[REG_S as usize - 8]));
        //println!("c {}", mutor.read_u32(mutor.registers[REG_S as usize - 4]));
        //println!("c {}", mutor.read_u32(mutor.registers[REG_S as usize]));
        //println!("c {}", mutor.read_u32(mutor.registers[REG_S as usize + 4]));
        //println!("c {}", mutor.read_u32(mutor.registers[REG_S as usize + 8]));
        //println!("c {}", mutor.read_u32(mutor.registers[REG_S as usize + 12]));
        //println!("{:?}", mutor.registers);
        //println!();
    }; "call a function at addr which returns via `ret`, adding a stack frame"; }
    instr INSTR_RET { INSTR_RET_STR = ret;  unsafe {
        let mutor = thread.mutator();
        //println!("{:?}", mutor.registers);
        //println!("r {}", mutor.read_u32(mutor.registers[REG_S as usize - 12]));
        //println!("r {}", mutor.read_u32(mutor.registers[REG_S as usize - 8]));
        //println!("r {}", mutor.read_u32(mutor.registers[REG_S as usize - 4]));
        //println!("r {}", mutor.read_u32(mutor.registers[REG_S as usize]));
        //println!("r {}", mutor.read_u32(mutor.registers[REG_S as usize + 4]));
        //println!("r {}", mutor.read_u32(mutor.registers[REG_S as usize + 8]));
        //println!("r {}", mutor.read_u32(mutor.registers[REG_S as usize + 12]));
        mutor.registers[REG_S as usize] = mutor.registers[REG_B as usize];
        let ret_i = mutor.read_u32(mutor.registers[REG_S as usize]);
        let base = mutor.read_u32(mutor.registers[REG_S as usize] + 4);
        mutor.registers[REG_I as usize] = ret_i;
        mutor.registers[REG_B as usize] = base;
        mutor.registers[REG_S as usize] -= 4;
        //println!("{:?}", mutor.registers);
        //println!();
    }; "return from a function, removing a stack frame"; }
    
    
    instr INSTR_JZ { INSTR_JZ_STR = jz; impl_jump!(thread jump a if FLAG_BIT_Z); "jump if FLAG_BIT_Z is set"; }
    instr INSTR_JNZ { INSTR_JNZ_STR = jnz; impl_jump!(thread jump a unless FLAG_BIT_Z); "jump if FLAG_BIT_Z is unset"; }
    instr INSTR_CLZF { INSTR_CLZF_STR = clzf; impl_jump!(thread clear FLAG_BIT_Z); "clear FLAG_BIT_Z"; }
    instr INSTR_JS { INSTR_JS_STR = js; impl_jump!(thread jump a if FLAG_BIT_S); "jump if FLAG_BIT_S is set"; }
    instr INSTR_JNS { INSTR_JNS_STR = jns; impl_jump!(thread jump a unless FLAG_BIT_S); "jump if FLAG_BIT_S is unset"; }
    instr INSTR_CLSF { INSTR_CLSF_STR = clsf; impl_jump!(thread clear FLAG_BIT_S); "clear FLAG_BIT_S"; }
    instr INSTR_JC { INSTR_JC_STR = jc; impl_jump!(thread jump a if FLAG_BIT_C); "jump if FLAG_BIT_C is set"; }
    instr INSTR_JNC { INSTR_JNC_STR = jnc; impl_jump!(thread jump a unless FLAG_BIT_C); "jump if FLAG_BIT_C is unset"; }
    instr INSTR_CLCF { INSTR_CLCF_STR = clcf; impl_jump!(thread clear FLAG_BIT_C); "FLAG_BIT_C"; }
    instr INSTR_JE { INSTR_JE_STR = je; impl_jump!(thread jump a if FLAG_BIT_E); "jump if FLAG_BIT_E is set"; }
    instr INSTR_JNE { INSTR_JNE_STR = jne; impl_jump!(thread jump a unless FLAG_BIT_E); "jump if FLAG_BIT_E is unset"; }
    instr INSTR_CLEF { INSTR_CLEF_STR = clef; impl_jump!(thread clear FLAG_BIT_E); "clear FLAG_BIT_E"; }
    instr INSTR_JL { INSTR_JL_STR = jl; impl_jump!(thread jump a if FLAG_BIT_L); "jump if FLAG_BIT_L is set"; }
    instr INSTR_JNL { INSTR_JNL_STR = jnl; impl_jump!(thread jump a unless FLAG_BIT_L); "jump if FLAG_BIT_L is unset"; }
    instr INSTR_CLLF { INSTR_CLLF_STR = cllf; impl_jump!(thread clear FLAG_BIT_L); "clear FLAG_BIT_L"; }

    instr INSTR_WRITE_STDOUT { INSTR_WRITET_STDOUT_STR = write_stdout; impl_func!(thread |a: u32| print!("{}", char::from_u32(a).unwrap_or_else(|| unsafe { thread.mutator().registers[REG_F as usize] |= FLAG_BIT_E; '�' })) => ()); "u32: print char to stdout, flushes on newline (\\n). Sets FLAG_BIT_E on invalid char"; }
    instr INSTR_FLUSH_STDOUT { INSTR_FLUSH_STDOUT_STR = flush_stdout; std::io::stdout().flush().unwrap_or_else(|_| unsafe { thread.mutator().registers[REG_F as usize] |= FLAG_BIT_E; }); "Flush stdout. Sets FLAG_BIT_E if errors while flushing."; }
    instr INSTR_READ_STDIN { INSTR_READ_STDIN_STR = read_stdin; impl_func!(thread || getch::Getch::new().getch().unwrap_or_else(|_| unsafe { thread.mutator().registers[REG_F as usize] |= FLAG_BIT_E; 0 }) as u32 => (r: u32 => [write to reg a])); "Wait for a char on stdin. Sets FLAG_BIT_E if errors while getting."; }

    // note: memory instructions follow the order convention of `instr source destination`
    instr INSTR_LD { INSTR_LD_STR = ld; impl_func!(thread |a: u32| thread.read_u32(a) => (r: u32 => [write to reg b])); "load source_addr dest_reg_or_stack"; }
    instr INSTR_ST { INSTR_ST_STR = st; impl_func!(thread |a: u32, b: u32| thread.write_u32(a, b) => ()); "store source_reg_stack_or_val dest_addr"; }
    instr INSTR_MOV { INSTR_MOV_STR = mov; impl_func!(thread |a: u32| a => (r: u32 => [write to reg b])); "move source_reg_stack_or_val dest_reg_or_stack"; }
    instr INSTR_SWAP { INSTR_SWAP_STR = swap; impl_func!(thread |a: u32 as ax, b: u32 as bx| (bx, ax) => (ar: u32 => [write to reg a], br: u32 => [write to reg b])); "swap source_reg dest_reg_or_stack"; }
    instr INSTR_LD8 { INSTR_LD8_STR = ld8; impl_func!(thread |a: u32| thread.read_u8(a) as u32 => (r: u32 => [write to reg b])); "load 8 bytes source_addr dest_reg_or_stack, zeroing upper 3 bytes of reg"; }
    instr INSTR_ST8 { INSTR_ST8_STR = st8; impl_func!(thread |a: u32, b: u32| thread.write_u8(a, b as u8) => ()); "store 8 bytes source_reg_stack_or_val dest_reg_or_stack, ignoring upper 3 bytes of reg"; }
    instr INSTR_DUP { INSTR_DUP_STR = dup; {
        unsafe {
            let mutor = thread.mutator();
            let v = mutor.read_u32(mutor.registers[REG_S as usize]);
            mutor.registers[REG_S as usize] += 4;
            thread.write_u32(mutor.registers[REG_S as usize], v);
        }
    }; "duplicate topmost stack element"; }
    instr INSTR_POP { INSTR_POP_STR = pop; unsafe { let mutor = thread.mutator();  mutor.registers[REG_S as usize] -= 4; }; "removes topmost stack element"; }
    instr INSTR_ROTD { INSTR_ROTD_STR = rotd; {
        unsafe {
            let mutor = thread.mutator();
            let a = mutor.read_u32(mutor.registers[REG_S as usize] - 8);
            let b = mutor.read_u32(mutor.registers[REG_S as usize] - 4);
            let c = mutor.read_u32(mutor.registers[REG_S as usize]);
            mutor.write_u32(mutor.registers[REG_S as usize] - 8, c);
            mutor.write_u32(mutor.registers[REG_S as usize] - 4, a);
            mutor.write_u32(mutor.registers[REG_S as usize], b);
        }
    }; "rotates-down the top stack elem 2 places, moving second and third one up each: (bottom) a b c (top) -> (bottom) c a b (top). opposite of rotu"; }
    instr INSTR_ROTU { INSTR_ROTU_STR = rotu; {
        unsafe {
            let mutor = thread.mutator();
            let a = mutor.read_u32(mutor.registers[REG_S as usize] - 8);
            let b = mutor.read_u32(mutor.registers[REG_S as usize] - 4);
            let c = mutor.read_u32(mutor.registers[REG_S as usize]);
            mutor.write_u32(mutor.registers[REG_S as usize] - 8, b);
            mutor.write_u32(mutor.registers[REG_S as usize] - 4, c);
            mutor.write_u32(mutor.registers[REG_S as usize], a);
        }
    }; "rotates-up the third stack elem 2 places, moving first and second down one each: (bottom) a b c (top) -> (bottom) b c a (top). opposite of rotd"; }

    instr INSTR_BREAKPOINT { INSTR_BREAKPOINT_STR = breakpoint; { 
        println!();
        for i in 40..50 {
            println!(" {:2} | {:10} | 0x{:08X} | {}", i, thread.registers[i], thread.registers[i], unsafe { std::mem::transmute::<u32, f32>(thread.registers[i]) }); 
        }
        std::io::stdin().read_line(&mut String::new()).unwrap(); 
    }; "debug breakpoint"; }
}