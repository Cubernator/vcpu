use super::*;

#[test]
fn works() {
    instruction_runs! {
        instr_alu!(XOR, T0, T1, T2),
        [
            T1 = 0b0101_0011_0010_0011_1111_0100_0110_1011_u32,
            T2 = 0b1110_1111_1001_0101_0111_1101_1010_0111_u32
        ] => [
            T0 = 0b1011_1100_1011_0110_1000_1001_1100_1100_u32
        ]
    }
}
