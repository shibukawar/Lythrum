// asm.rs
pub fn hlt() {
    unsafe {
        asm!("hlt");
    }
}

pub fn load_eflags() -> i32 {
    let result: i32;
    unsafe {
        asm!("PUSHFD");
        asm!("POP {0}", out(reg) result);
    }
    result
}

pub fn store_eflags(flags: i32) {
    unsafe {
        // {0} は展開されてコンパイラが適当に選んだレジスタ名に置き換わる
        // そこにflagsを束縛する
        asm!("PUSH {0}", in(reg) flags);
        asm!("POPFD");
    }
}

pub fn cli() {
    unsafe {
        asm!("CLI");
    }
}


pub fn sti() {
    unsafe {
        asm!("STI");
    }
}

pub fn stihlt() {
    unsafe {
        asm!("STI
              HLT");
    }
}

// outに使うときでなおかつ初期化しているときはmutにする必要がある
pub fn out8(mut port: u32, mut data: u8) {
    unsafe {
        // OUT dx,al で命令は完成している(ポートにつながっているデバイスにデータを送る)
        // dx, alはそれぞれ具体的なレジスタをさしていて、それぞれにどの変数を束縛するかを後から指定してやる必要がある
        asm!("OUT dx,al", inout("dx") port, inout("al") data);
    }
}

pub fn in8(port: u32) -> u8 {
    let mut data: u8 = 0;
    unsafe {
        asm!("IN al,dx", out("al") data, in("edx") port);
    }
    data
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct Dtr {
    limit: i16,
    base: i32,
}

pub fn load_gdtr(limit: i32, adr: i32) {
    unsafe {
        asm!("LGDT [{}]", in(reg) &Dtr { limit: limit as i16, base: adr } );
    }
}

pub fn load_idtr(limit: i32, adr: i32) {
    unsafe {
        asm!("LIDT [{}]", in(reg) &Dtr { limit: limit as i16, base: adr } );
    }
}

#[macro_export]
macro_rules! handler {
    ($name: ident) => {{ 
        pub extern "C" fn wrapper() {
            unsafe {
                asm!("PUSH ES
                      PUSH DS
                      PUSHAD
                      MOV EAX,ESP
                      PUSH EAX
                      MOV AX,SS
                      MOV DS,AX
                      MOV ES,AX");
                asm!("CALL {}", in(reg) $name as extern "C" fn()); // [{}] だと命令列をアドレスだと判断してそこをみにいっちゃう
                asm!("POP EAX
                      POPAD
                      POP DS
                      POP ES
                      IRETD");
            }
        }
        wrapper
    }}
}