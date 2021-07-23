import { LitElement, html, css} from "lit-element";
import { styleMap } from "lit-html/directives/style-map";

const LINE_HEIGHT = 16

class Program extends LitElement {


    static get properties() {
        return {
            data: {attribute: false},
            play: {attribute: false},
            pc: {attribute: false},
            dmg: {attribute: false},
            breakpoints: {attribute: false}
        }
    }

    static get styles() {
        return css`
            :host {
                background-color: #1D1E23;
                margin-left: 10px;
                width: 100%;
                padding: 5px;
                border-radius: 5px;
            }

            .program {
                font-size: 14px;
                height: 200px;
                overflow-y: scroll;
                margin-top: 10px;
                line-height: 16px;
                margin-bottom: 0px;
            }
            :host > p {
                margin-top: 0px;
            }

            .program::-webkit-scrollbar {
                width: 10px;               /* width of the entire scrollbar */
            }

            .program::-webkit-scrollbar-track {
                background: rgb(37, 39, 49);        /* color of the tracking area */
            }

            .program::-webkit-scrollbar-thumb {
                background-color: var(--white);    /* color of the scroll thumb */
                border-radius: 20px;       /* roundness of the scroll thumb */
            }

            pre {
                cursor: pointer;
            }
        `
    }

    constructor() {
        super();
        this.play = false;
        this.pc = 0;
        this.linesBeforePC = 0;
        this.breakpoints = [
            0x0100,
        ];
        if (!this.hasAttribute('tabindex')) {
            this.setAttribute('tabindex', 0);
        }
        this.memChange = false;
    }

    connectedCallback() {
        super.connectedCallback();
        this.dmg.update_breakpoints(this.breakpoints);
    }

    updated(changed) {
        let pWindow = this.shadowRoot.querySelector('.program');
        if (!pWindow) return;

        pWindow.scroll(0, (this.linesBeforePC) * LINE_HEIGHT);
    }

    _handleAddBreak(e) {
        const pre = this.shadowRoot.querySelector('pre');
        let scrollOffset = pre.scrollTop;
        let windowOffset = pre.offsetTop;
        let mouseOffset = e.clientY;
        let mouseHeight = mouseOffset - windowOffset;
        let line = Math.floor((scrollOffset + mouseHeight) / LINE_HEIGHT)
        let instrs = pre.innerText;
        let chosen_instr = instrs.split('\n').filter(s => s.length > 0)[line];
        let breakpoint = Number.parseInt(chosen_instr.split(':')[0].substr(2), 16);
        console.log(breakpoint);
        let existing = this.breakpoints.indexOf(breakpoint);
        if (existing > -1) {
            this.breakpoints.splice(existing, 1);
            this.breakpoints = [...this.breakpoints];
        } else {
            this.breakpoints = [...this.breakpoints, breakpoint];
        }
        this.memChange = false;
        this.dmg.update_breakpoints(this.breakpoints);
    }

    render() {
        if (this.data.length === 0) {
            return html`
            <p>Program</p>
        `;
        }
        if (!this.play) this.decompiler(); // populates instrs
        return html`
            <p>Program</p>
            <pre @click=${this._handleAddBreak} class="program">${this.instrs}</pre>
        `;
    }

    decompiler() {
        this.linesBeforePC = 0;
        this.instrs = [];
        this.instrs[0] = "";
        this.instrs[1] = "";
        this.instrs[2] = "";
        let pc = 0;
        while (pc < 65535) {
            let size = this.decompileLine(pc);
            pc += size;
            if (pc < this.pc) {
                this.linesBeforePC += 1;
            }
        }
    }

    decompileLine(pc) {
        let op = this.data[pc];
        let size = this.opSize(op);
        let spc = this.hex4(pc);
        let has_break = this.breakpoints.find(b => b === pc);
        let icon = has_break ? `●` : ' ';


        let s = icon;
        if (size === 1) {
            s += ` ${spc}: ${this.opToInstr(op)}`;
        } else if (size === 2) {
            s += ` ${spc}: ${this.opToInstr(op, this.data[pc + 1])}`;
        } else if (size === 3) {
            s += ` ${spc}: ${this.opToInstr(op, this.data[pc + 1], this.data[pc + 2])}`;
        }
        if (this.pc === pc) {
            let style = styleMap({
                "background-color": 'var(--secondary)', 
                "width": '100%', 
                "margin": '0px', 
            });
            this.instrs[1] = html`<p style="${style}">${s}</p>`;
        } else if (this.pc > pc) {
            this.instrs[0] += s + '\n';
        } else if (this.pc < pc) {
            this.instrs[2] += s + '\n';
        }
        return this.opSize(op);
    }

    hex(n) {
        if (!n) return '00';
        let s = n.toString(16).toUpperCase();
        return s.length === 1 ? `0${s}`
             : s;
    }

    hex4(n) {
        let s = n.toString(16).toUpperCase();
        return s.length === 1 ? `000${s}`
             : s.length === 2 ? `00${s}`
             : s.length === 3 ? `0${s}`
             : s
    }

    opToInstr(op1, op2, op3) {
        let oop2 = op2;
        if (op2) {op2 = this.hex(op2)};
        if (op3) {op3 = this.hex(op3)};

        switch (op1) {
            case 0x00: return 'NOP';
            case 0x01: return `LD BC, ${op3}${op2}`;
            case 0x02: return `LD (BC), A`;
            case 0x03: return `INC BC`;
            case 0x04: return `INC B`;
            case 0x05: return `DEC B`;
            case 0x06: return `LD B, ${op2}`;
            case 0x07: return `RLCA`
            case 0x08: return `LD (${op3}${op2}), SP`;
            case 0x09: return `ADD HL, BC`;
            case 0x0A: return `LD A, (BC)`;
            case 0x0B: return `DEC BC`;
            case 0x0C: return `INC C`;
            case 0x0D: return `DEC C`;
            case 0x0E: return `LD C, ${op2}`;
            case 0x0F: return `RRCA`;

            case 0x10: return 'STOP';
            case 0x11: return `LD DE, ${op3}${op2}`;
            case 0x12: return `LD (DE), A`;
            case 0x13: return `INC DE`;
            case 0x14: return `INC D`;
            case 0x15: return `DEC D`;
            case 0x16: return `LD D, ${op2}`;
            case 0x17: return `RLA`
            case 0x18: return `JR ${op2}`;
            case 0x19: return `ADD HL, DE`;
            case 0x1A: return `LD A, (DE)`;
            case 0x1B: return `DEC DE`;
            case 0x1C: return `INC E`;
            case 0x1D: return `DEC E`;
            case 0x1E: return `LD E, ${op2}`;
            case 0x1F: return `RRA`;

            case 0x20: return `JR NZ, ${op2}`;
            case 0x21: return `LD HL, ${op3}${op2}`;
            case 0x22: return `LD (HL+), A`;
            case 0x23: return `INC HL`;
            case 0x24: return `INC H`;
            case 0x25: return `DEC H`;
            case 0x26: return `LD H, ${op2}`;
            case 0x27: return `DAA`
            case 0x28: return `JR Z, ${op2}`;
            case 0x29: return `ADD HL, HL`;
            case 0x2A: return `LD A, (HL+)`;
            case 0x2B: return `DEC HL`;
            case 0x2C: return `INC L`;
            case 0x2D: return `DEC L`;
            case 0x2E: return `LD L, ${op2}`;
            case 0x2F: return `CPL`;

            case 0x30: return `JR NC, ${op2}`;
            case 0x31: return `LD SP, ${op3}${op2}`;
            case 0x32: return `LD (HL-), A`;
            case 0x33: return `INC SP`;
            case 0x34: return `INC (HL)`;
            case 0x35: return `DEC (HL)`;
            case 0x36: return `LD (HL), ${op2}`;
            case 0x37: return `SCF`
            case 0x38: return `JR C, ${op2}`;
            case 0x39: return `ADD HL, SP`;
            case 0x3A: return `LD A, (HL-)`;
            case 0x3B: return `DEC SP`;
            case 0x3C: return `INC A`;
            case 0x3D: return `DEC A`;
            case 0x3E: return `LD A, ${op2}`;
            case 0x3F: return `CCF`;

            case 0x40: return `LD B, B`;
            case 0x41: return `LD B, C`;
            case 0x42: return `LD B, D`;
            case 0x43: return `LD B, E`;
            case 0x44: return `LD B, H`;
            case 0x45: return `LD B, L`;
            case 0x46: return `LD B, (HL)`;
            case 0x47: return `LD B, A`;
            case 0x48: return `LD C, B`;
            case 0x49: return `LD C, C`;
            case 0x4A: return `LD C, D`;
            case 0x4B: return `LD C, E`;
            case 0x4C: return `LD C, H`;
            case 0x4D: return `LD C, L`;
            case 0x4E: return `LD C, (HL)`;
            case 0x4F: return `LD C, A`;

            case 0x50: return `LD D, B`;
            case 0x51: return `LD D, C`;
            case 0x52: return `LD D, D`;
            case 0x53: return `LD D, E`;
            case 0x54: return `LD D, H`;
            case 0x55: return `LD D, L`;
            case 0x56: return `LD D, (HL)`;
            case 0x57: return `LD D, A`;
            case 0x58: return `LD E, B`;
            case 0x59: return `LD E, C`;
            case 0x5A: return `LD E, D`;
            case 0x5B: return `LD E, E`;
            case 0x5C: return `LD E, H`;
            case 0x5D: return `LD E, L`;
            case 0x5E: return `LD E, (HL)`;
            case 0x5F: return `LD E, A`;

            case 0x60: return `LD H, B`;
            case 0x61: return `LD H, C`;
            case 0x62: return `LD H, D`;
            case 0x63: return `LD H, E`;
            case 0x64: return `LD H, H`;
            case 0x65: return `LD H, L`;
            case 0x66: return `LD H, (HL)`;
            case 0x67: return `LD H, A`;
            case 0x68: return `LD L, B`;
            case 0x69: return `LD L, C`;
            case 0x6A: return `LD L, D`;
            case 0x6B: return `LD L, E`;
            case 0x6C: return `LD L, H`;
            case 0x6D: return `LD L, L`;
            case 0x6E: return `LD L, (HL)`;
            case 0x6F: return `LD L, A`;

            case 0x70: return `LD (HL), B`;
            case 0x71: return `LD (HL), C`;
            case 0x72: return `LD (HL), D`;
            case 0x73: return `LD (HL), E`;
            case 0x74: return `LD (HL), H`;
            case 0x75: return `LD (HL), L`;
            case 0x76: return `HALT`;
            case 0x77: return `LD (HL), A`;
            case 0x78: return `LD A, B`;
            case 0x79: return `LD A, C`;
            case 0x7A: return `LD A, D`;
            case 0x7B: return `LD A, E`;
            case 0x7C: return `LD A, H`;
            case 0x7D: return `LD A, L`;
            case 0x7E: return `LD A, (HL)`;
            case 0x7F: return `LD A, A`;

            case 0x80: return `ADD A, B`;
            case 0x81: return `ADD A, C`;
            case 0x82: return `ADD A, D`;
            case 0x83: return `ADD A, E`;
            case 0x84: return `ADD A, H`;
            case 0x85: return `ADD A, L`;
            case 0x86: return `ADD A, (HL)`;
            case 0x87: return `ADD A, A`;
            case 0x88: return `ADC A, B`;
            case 0x89: return `ADC A, C`;
            case 0x8A: return `ADC A, D`;
            case 0x8B: return `ADC A, E`;
            case 0x8C: return `ADC A, H`;
            case 0x8D: return `ADC A, L`;
            case 0x8E: return `ADC A, (HL)`;
            case 0x8F: return `ADC A, A`;

            case 0x90: return `SUB A, B`;
            case 0x91: return `SUB A, C`;
            case 0x92: return `SUB A, D`;
            case 0x93: return `SUB A, E`;
            case 0x94: return `SUB A, H`;
            case 0x95: return `SUB A, L`;
            case 0x96: return `SUB A, (HL)`;
            case 0x97: return `SUB A, A`;
            case 0x98: return `SBC A, B`;
            case 0x99: return `SBC A, C`;
            case 0x9A: return `SBC A, D`;
            case 0x9B: return `SBC A, E`;
            case 0x9C: return `SBC A, H`;
            case 0x9D: return `SBC A, L`;
            case 0x9E: return `SBC A, (HL)`;
            case 0x9F: return `SBC A, A`;

            case 0xA0: return `AND A, B`;
            case 0xA1: return `AND A, C`;
            case 0xA2: return `AND A, D`;
            case 0xA3: return `AND A, E`;
            case 0xA4: return `AND A, H`;
            case 0xA5: return `AND A, L`;
            case 0xA6: return `AND A, (HL)`;
            case 0xA7: return `AND A, A`;
            case 0xA8: return `XOR A, B`;
            case 0xA9: return `XOR A, C`;
            case 0xAA: return `XOR A, D`;
            case 0xAB: return `XOR A, E`;
            case 0xAC: return `XOR A, H`;
            case 0xAD: return `XOR A, L`;
            case 0xAE: return `XOR A, (HL)`;
            case 0xAF: return `XOR A, A`;

            case 0xB0: return `OR A, B`;
            case 0xB1: return `OR A, C`;
            case 0xB2: return `OR A, D`;
            case 0xB3: return `OR A, E`;
            case 0xB4: return `OR A, H`;
            case 0xB5: return `OR A, L`;
            case 0xB6: return `OR A, (HL)`;
            case 0xB7: return `OR A, A`;
            case 0xB8: return `CP A, B`;
            case 0xB9: return `CP A, C`;
            case 0xBA: return `CP A, D`;
            case 0xBB: return `CP A, E`;
            case 0xBC: return `CP A, H`;
            case 0xBD: return `CP A, L`;
            case 0xBE: return `CP A, (HL)`;
            case 0xBF: return `CP A, A`;

            case 0xC0: return `RET NZ`;
            case 0xC1: return `POP BC`;
            case 0xC2: return `JP NZ, ${op3}${op2}`;
            case 0xC3: return `JP ${op3}${op2}`;
            case 0xC4: return `CALL NZ ${op3}${op2}`;
            case 0xC5: return `PUSH BC`;
            case 0xC6: return `ADD A, ${op2}`;
            case 0xC7: return `RST 0`;
            case 0xC8: return `RET Z`;
            case 0xC9: return `RET`;
            case 0xCA: return `JP Z, ${op3}${op2}`;
            case 0xCB: 
                switch (oop2) {
                    case 0x00: return `RLC B`;
                    case 0x01: return `RLC C`;
                    case 0x02: return `RLC D`;
                    case 0x03: return `RLC E`;
                    case 0x04: return `RLC H`;
                    case 0x05: return `RLC L`;
                    case 0x06: return `RLC (HL)`;
                    case 0x07: return `RLC A`;
                    case 0x08: return `RRC B`;
                    case 0x09: return `RRC C`;
                    case 0x0A: return `RRC D`;
                    case 0x0B: return `RRC E`;
                    case 0x0C: return `RRC H`;
                    case 0x0D: return `RRC L`;
                    case 0x0E: return `RRC (HL)`;
                    case 0x0F: return `RRC A`;

                    case 0x10: return `RL B`;
                    case 0x11: return `RL C`;
                    case 0x12: return `RL D`;
                    case 0x13: return `RL E`;
                    case 0x14: return `RL H`;
                    case 0x15: return `RL L`;
                    case 0x16: return `RL (HL)`;
                    case 0x17: return `RL A`;
                    case 0x18: return `RR B`;
                    case 0x19: return `RR C`;
                    case 0x1A: return `RR D`;
                    case 0x1B: return `RR E`;
                    case 0x1C: return `RR H`;
                    case 0x1D: return `RR L`;
                    case 0x1E: return `RR (HL)`;
                    case 0x1F: return `RR A`;

                    case 0x20: return `SLA B`;
                    case 0x21: return `SLA C`;
                    case 0x22: return `SLA D`;
                    case 0x23: return `SLA E`;
                    case 0x24: return `SLA H`;
                    case 0x25: return `SLA L`;
                    case 0x26: return `SLA (HL)`;
                    case 0x27: return `SLA A`;
                    case 0x28: return `SRA B`;
                    case 0x29: return `SRA C`;
                    case 0x2A: return `SRA D`;
                    case 0x2B: return `SRA E`;
                    case 0x2C: return `SRA H`;
                    case 0x2D: return `SRA L`;
                    case 0x2E: return `SRA (HL)`;
                    case 0x2F: return `SRA A`;

                    case 0x30: return `SWAP B`;
                    case 0x31: return `SWAP C`;
                    case 0x32: return `SWAP D`;
                    case 0x33: return `SWAP E`;
                    case 0x34: return `SWAP H`;
                    case 0x35: return `SWAP L`;
                    case 0x36: return `SWAP (HL)`;
                    case 0x37: return `SWAP A`;
                    case 0x38: return `SWAP B`;
                    case 0x39: return `SWAP C`;
                    case 0x3A: return `SWAP D`;
                    case 0x3B: return `SWAP E`;
                    case 0x3C: return `SWAP H`;
                    case 0x3D: return `SWAP L`;
                    case 0x3E: return `SWAP (HL)`;
                    case 0x3F: return `SWAP A`;

                    case 0x40: return `BIT 0, B`;
                    case 0x41: return `BIT 0, C`;
                    case 0x42: return `BIT 0, D`;
                    case 0x43: return `BIT 0, E`;
                    case 0x44: return `BIT 0, H`;
                    case 0x45: return `BIT 0, L`;
                    case 0x46: return `BIT 0, (HL)`;
                    case 0x47: return `BIT 0, A`;
                    case 0x48: return `BIT 1, B`;
                    case 0x49: return `BIT 1, C`;
                    case 0x4A: return `BIT 1, D`;
                    case 0x4B: return `BIT 1, E`;
                    case 0x4C: return `BIT 1, H`;
                    case 0x4D: return `BIT 1, L`;
                    case 0x4E: return `BIT 1, (HL)`;
                    case 0x4F: return `BIT 1, A`;

                    case 0x50: return `BIT 2, B`;
                    case 0x51: return `BIT 2, C`;
                    case 0x52: return `BIT 2, D`;
                    case 0x53: return `BIT 2, E`;
                    case 0x54: return `BIT 2, H`;
                    case 0x55: return `BIT 2, L`;
                    case 0x56: return `BIT 2, (HL)`;
                    case 0x57: return `BIT 2, A`;
                    case 0x58: return `BIT 3, B`;
                    case 0x59: return `BIT 3, C`;
                    case 0x5A: return `BIT 3, D`;
                    case 0x5B: return `BIT 3, E`;
                    case 0x5C: return `BIT 3, H`;
                    case 0x5D: return `BIT 3, L`;
                    case 0x5E: return `BIT 3, (HL)`;
                    case 0x5F: return `BIT 3, A`;

                    case 0x60: return `BIT 4, B`;
                    case 0x61: return `BIT 4, C`;
                    case 0x62: return `BIT 4, D`;
                    case 0x63: return `BIT 4, E`;
                    case 0x64: return `BIT 4, H`;
                    case 0x65: return `BIT 4, L`;
                    case 0x66: return `BIT 4, (HL)`;
                    case 0x67: return `BIT 4, A`;
                    case 0x68: return `BIT 5, B`;
                    case 0x69: return `BIT 5, C`;
                    case 0x6A: return `BIT 5, D`;
                    case 0x6B: return `BIT 5, E`;
                    case 0x6C: return `BIT 5, H`;
                    case 0x6D: return `BIT 5, L`;
                    case 0x6E: return `BIT 5, (HL)`;
                    case 0x6F: return `BIT 5, A`;

                    case 0x70: return `BIT 6, B`;
                    case 0x71: return `BIT 6, C`;
                    case 0x72: return `BIT 6, D`;
                    case 0x73: return `BIT 6, E`;
                    case 0x74: return `BIT 6, H`;
                    case 0x75: return `BIT 6, L`;
                    case 0x76: return `BIT 6, (HL)`;
                    case 0x77: return `BIT 6, A`;
                    case 0x78: return `BIT 7, B`;
                    case 0x79: return `BIT 7, C`;
                    case 0x7A: return `BIT 7, D`;
                    case 0x7B: return `BIT 7, E`;
                    case 0x7C: return `BIT 7, H`;
                    case 0x7D: return `BIT 7, L`;
                    case 0x7E: return `BIT 7, (HL)`;
                    case 0x7F: return `BIT 7, A`;

                    case 0x80: return `RES 0, B`;
                    case 0x81: return `RES 0, C`;
                    case 0x82: return `RES 0, D`;
                    case 0x83: return `RES 0, E`;
                    case 0x84: return `RES 0, H`;
                    case 0x85: return `RES 0, L`;
                    case 0x86: return `RES 0, (HL)`;
                    case 0x87: return `RES 0, A`;
                    case 0x88: return `RES 1, B`;
                    case 0x89: return `RES 1, C`;
                    case 0x8A: return `RES 1, D`;
                    case 0x8B: return `RES 1, E`;
                    case 0x8C: return `RES 1, H`;
                    case 0x8D: return `RES 1, L`;
                    case 0x8E: return `RES 1, (HL)`;
                    case 0x8F: return `RES 1, A`;

                    case 0x90: return `RES 2, B`;
                    case 0x91: return `RES 2, C`;
                    case 0x92: return `RES 2, D`;
                    case 0x93: return `RES 2, E`;
                    case 0x94: return `RES 2, H`;
                    case 0x95: return `RES 2, L`;
                    case 0x96: return `RES 2, (HL)`;
                    case 0x97: return `RES 2, A`;
                    case 0x98: return `RES 3, B`;
                    case 0x99: return `RES 3, C`;
                    case 0x9A: return `RES 3, D`;
                    case 0x9B: return `RES 3, E`;
                    case 0x9C: return `RES 3, H`;
                    case 0x9D: return `RES 3, L`;
                    case 0x9E: return `RES 3, (HL)`;
                    case 0x9F: return `RES 3, A`;

                    case 0xA0: return `RES 4, B`;
                    case 0xA1: return `RES 4, C`;
                    case 0xA2: return `RES 4, D`;
                    case 0xA3: return `RES 4, E`;
                    case 0xA4: return `RES 4, H`;
                    case 0xA5: return `RES 4, L`;
                    case 0xA6: return `RES 4, (HL)`;
                    case 0xA7: return `RES 4, A`;
                    case 0xA8: return `RES 5, B`;
                    case 0xA9: return `RES 5, C`;
                    case 0xAA: return `RES 5, D`;
                    case 0xAB: return `RES 5, E`;
                    case 0xAC: return `RES 5, H`;
                    case 0xAD: return `RES 5, L`;
                    case 0xAE: return `RES 5, (HL)`;
                    case 0xAF: return `RES 5, A`;

                    case 0xB0: return `RES 6, B`;
                    case 0xB1: return `RES 6, C`;
                    case 0xB2: return `RES 6, D`;
                    case 0xB3: return `RES 6, E`;
                    case 0xB4: return `RES 6, H`;
                    case 0xB5: return `RES 6, L`;
                    case 0xB6: return `RES 6, (HL)`;
                    case 0xB7: return `RES 6, A`;
                    case 0xB8: return `RES 7, B`;
                    case 0xB9: return `RES 7, C`;
                    case 0xBA: return `RES 7, D`;
                    case 0xBB: return `RES 7, E`;
                    case 0xBC: return `RES 7, H`;
                    case 0xBD: return `RES 7, L`;
                    case 0xBE: return `RES 7, (HL)`;
                    case 0xBF: return `RES 7, A`;

                    case 0xC0: return `SET 0, B`;
                    case 0xC1: return `SET 0, C`;
                    case 0xC2: return `SET 0, D`;
                    case 0xC3: return `SET 0, E`;
                    case 0xC4: return `SET 0, H`;
                    case 0xC5: return `SET 0, L`;
                    case 0xC6: return `SET 0, (HL)`;
                    case 0xC7: return `SET 0, A`;
                    case 0xC8: return `SET 1, B`;
                    case 0xC9: return `SET 1, C`;
                    case 0xCA: return `SET 1, D`;
                    case 0xCB: return `SET 1, E`;
                    case 0xCC: return `SET 1, H`;
                    case 0xCD: return `SET 1, L`;
                    case 0xCE: return `SET 1, (HL)`;
                    case 0xCF: return `SET 1, A`;

                    case 0xD0: return `SET 2, B`;
                    case 0xD1: return `SET 2, C`;
                    case 0xD2: return `SET 2, D`;
                    case 0xD3: return `SET 2, E`;
                    case 0xD4: return `SET 2, H`;
                    case 0xD5: return `SET 2, L`;
                    case 0xD6: return `SET 2, (HL)`;
                    case 0xD7: return `SET 2, A`;
                    case 0xD8: return `SET 3, B`;
                    case 0xD9: return `SET 3, C`;
                    case 0xDA: return `SET 3, D`;
                    case 0xDB: return `SET 3, E`;
                    case 0xDC: return `SET 3, H`;
                    case 0xDD: return `SET 3, L`;
                    case 0xDE: return `SET 3, (HL)`;
                    case 0xDF: return `SET 3, A`;

                    case 0xE0: return `SET 4, B`;
                    case 0xE1: return `SET 4, C`;
                    case 0xE2: return `SET 4, D`;
                    case 0xE3: return `SET 4, E`;
                    case 0xE4: return `SET 4, H`;
                    case 0xE5: return `SET 4, L`;
                    case 0xE6: return `SET 4, (HL)`;
                    case 0xE7: return `SET 4, A`;
                    case 0xE8: return `SET 5, B`;
                    case 0xE9: return `SET 5, C`;
                    case 0xEA: return `SET 5, D`;
                    case 0xEB: return `SET 5, E`;
                    case 0xEC: return `SET 5, H`;
                    case 0xED: return `SET 5, L`;
                    case 0xEE: return `SET 5, (HL)`;
                    case 0xEF: return `SET 5, A`;

                    case 0xF0: return `SET 6, B`;
                    case 0xF1: return `SET 6, C`;
                    case 0xF2: return `SET 6, D`;
                    case 0xF3: return `SET 6, E`;
                    case 0xF4: return `SET 6, H`;
                    case 0xF5: return `SET 6, L`;
                    case 0xF6: return `SET 6, (HL)`;
                    case 0xF7: return `SET 6, A`;
                    case 0xF8: return `SET 7, B`;
                    case 0xF9: return `SET 7, C`;
                    case 0xFA: return `SET 7, D`;
                    case 0xFB: return `SET 7, E`;
                    case 0xFC: return `SET 7, H`;
                    case 0xFD: return `SET 7, L`;
                    case 0xFE: return `SET 7, (HL)`;
                    case 0xFF: return `SET 7, A`;
                }
            case 0xCC: return `CALL Z, ${op3}${op2}`;
            case 0xCD: return `CALL ${op3}${op2}`;
            case 0xCE: return `ADC A, ${op2}`;
            case 0xCF: return `RST 1`;

            case 0xD0: return `RET NC`;
            case 0xD1: return `POP DE`;
            case 0xD2: return `JP NC, ${op3}${op2}`;
            case 0xD3: return `???`;
            case 0xD4: return `CALL NC ${op3}${op2}`;
            case 0xD5: return `PUSH DE`;
            case 0xD6: return `SUB A, ${op2}`;
            case 0xD7: return `RST 2`;
            case 0xD8: return `RET C`;
            case 0xD9: return `RETI`;
            case 0xDA: return `JP C, ${op3}${op2}`;
            case 0xDB: return `???`;
            case 0xDC: return `CALL C, ${op3}${op2}`;
            case 0xDD: return `???`;
            case 0xDE: return `SBC A, ${op2}`;
            case 0xDF: return `RST 3`;

            case 0xE0: return `LD (0xFF + ${op2}), A`;
            case 0xE1: return `POP HL`;
            case 0xE2: return `LD (C), A`;
            case 0xE3: return `???`;
            case 0xE4: return `???`;
            case 0xE5: return `PUSH HL`;
            case 0xE6: return `AND A, ${op2}`;
            case 0xE7: return `RST 4`;
            case 0xE8: return `ADD SP, ${op2}`;
            case 0xE9: return `JP HL`;
            case 0xEA: return `LD (${op3}${op2}), A`;
            case 0xEB: return `???`;
            case 0xEC: return `???`;
            case 0xED: return `???`;
            case 0xEE: return `XOR A, ${op2}`;
            case 0xEF: return `RST 5`;

            case 0xF0: return `LD A, (0xFF + ${op2})`;
            case 0xF1: return `POP AF`;
            case 0xF2: return `LD A, (C)`;
            case 0xF3: return `DI`;
            case 0xF4: return `???`;
            case 0xF5: return `PUSH AF`;
            case 0xF6: return `OR A, ${op2}`;
            case 0xF7: return `RST 6`;
            case 0xF8: return `LD HL, SP+${op2}`;
            case 0xF9: return `LD SP, HL`;
            case 0xFA: return `LD A, (${op3}${op2})`;
            case 0xFB: return `EI`;
            case 0xFC: return `???`;
            case 0xFD: return `???`;
            case 0xFE: return `CP A, ${op2}`;
            case 0xFF: return `RST 7`;
            
        } 
    }

    opSize(op) {
        switch (op) {
            // 2 byte codes
            case 0x06: 
            case 0x0E: 
            case 0x10: 
            case 0x16: 
            case 0x1E: 
            case 0x20: 
            case 0x26: 
            case 0x28: 
            case 0x2E: 
            case 0x30: 
            case 0x38: 
            case 0x3E: 
            case 0xC6: 
            case 0xCB: 
            case 0xCE:
            case 0xD6: 
            case 0xDE: 
            case 0xE0: 
            case 0xE6: 
            case 0xE8: 
            case 0xEE: 
            case 0xF0: 
            case 0xF6: 
            case 0xF8: 
            case 0xFE: 
                return 2;
            case 0x01:
            case 0x08:
            case 0x11:
            case 0x21:
            case 0x31:
            case 0x31:
            case 0xC2:
            case 0xC3:
            case 0xC4:
            case 0xCA:
            case 0xCC:
            case 0xCD:
            case 0xD2:
            case 0xD4:
            case 0xDA:
            case 0xDC:
            case 0xEA:
            case 0xEA:
            case 0xFA:
                return 3;
            default:
                return 1;
        }
    }

}

customElements.define('program-debug', Program);