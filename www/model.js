import { Emu } from "dmg-emu";

let dmg = Emu.new();

export default {
    cpu: {
        AF: 0,
        BC: 0,
        DE: 0,
        HL: 0,
        SP: 0,
        PC: 0
    },
    mem: [],
    breakpoints: [],
    dmg
}