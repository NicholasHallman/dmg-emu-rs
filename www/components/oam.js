import { LitElement, html, css } from "lit-element";

class OAM extends LitElement {

    static get properties() {
        return {data: {attribute: false}}
    }

    static get styles() {
        return css`
            :host{
                display: grid;
                position: relative;
                background-color: #1D1E23;
                padding: 5px;
                border-radius: 5px;
                margin-top: 10px;
                width: calc(100% - 10px);
                grid-template-areas:
                    "title . . . . . . ."
                    "o00 o01 o02 o03 o04 o05 o06 o07"
                    "o10 o11 o12 o13 o14 o15 o16 o17"
                    "o20 o21 o22 o23 o24 o25 o26 o27"
                    "o30 o31 o32 o33 o34 o35 o36 o37"
                    "o40 o41 o42 o43 o44 o45 o46 o47"
            }
        `
    } 

    renderTile(n) {
        let x = n % 8;
        let y = Math.floor(n / 8);
        let gridName = `o${y}${x}`;
        let o_addr = 0xFE00 + (n * 4);
        let o_y = this.data[o_addr];
        let o_x = this.data[o_addr + 1];
        let o_t = this.data[o_addr + 2];
        let o_a = this.data[o_addr + 3];
        return html`
            <div style="grid-area: ${gridName};">
                <canvas width="8" height="8" style="width: 84px; height: 84px;"></canvas>
                <div style="display: flex; flex-direction: column;">
                    <span>X: ${o_x}</span>
                    <span>Y: ${o_y}</span>
                    <span>T: ${o_t}</span>
                    <span>A: ${o_a}</span>
                </div>
            </div>
        `
    }

    render() {
        return html`
            <p style="grid-area: title">OAM</p>
            ${Array(40).fill(0).map((_, i) => this.renderTile(i))}
        `
    }

}

customElements.define('oam-debug', OAM)