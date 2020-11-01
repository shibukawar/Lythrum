// asm.rs
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

// outに使うときでなおかつ初期化しているときはmutにする必要がある
pub fn out8(mut port: u32, mut data: u8) {
    unsafe {
        // OUT dx,al で命令は完成している(ポートにつながっているデバイスにデータを送る)
        // dx, alはそれぞれ具体的なレジスタをさしていて、それぞれにどの変数を束縛するかを後から指定してやる必要がある
        asm!("OUT dx,al", inout("dx") port, inout("al") data);
    }
}