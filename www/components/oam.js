import { LitElement, html, css } from "lit-element";

const WHITE = [0XFF, 0XFF, 0XFF, 0XFF];
const LGREY = [0XAB, 0XAB, 0XAB, 0XFF];
const DGREY = [0X67, 0X67, 0X67, 0XFF];
const BLACK = [0X12, 0X12, 0X12, 0XFF];
const COLORS = [
    WHITE,
    LGREY,
    DGREY,
    BLACK
]

const range = (n, predicate) => Array(n).fill(0).forEach((_, i) => predicate(i));
const mapRange = (n, predicate) => Array(n).fill(0).map((_, i) => predicate(i));

const flatMapRange = (n, predicate) => Array(n).fill(0).flatMap((_, i) => predicate(i));
class OAM extends LitElement {

    static get properties() {
        return {
            data: {attribute: false},
            play: {attribute: false}
        }
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
                    "o00 o01 o02 o03"
                    "o10 o11 o12 o13"
                    "o20 o21 o22 o23"
                    "o30 o31 o32 o33"
                    "o40 o41 o42 o43"
                    "o50 o51 o52 o53"
                    "o60 o61 o62 o63"
                    "o70 o71 o72 o73"
                    "o80 o81 o82 o83"
                    "o90 o91 o92 o93"
            }

            canvas {
                image-rendering: -moz-crisp-edges;
                image-rendering: -webkit-crisp-edges;
                image-rendering: pixelated;
                image-rendering: crisp-edges;
            }

            .o-card {
                border: solid 1px var(--primary);
            }

            .o-card span {
                width: 50%;
                font-size: 14px;
            }

        `
    }

    updated() {
        if (this.play) return;
        let ctxs = Array.from(this.shadowRoot.querySelectorAll('canvas')).map(c => c.getContext('2d'));

        ctxs.forEach((ctx, i) => {
            let tileId = this.data[(0xFE00 + (i * 4)) + 2];
            let data = new Uint8ClampedArray(flatMapRange(8, (j) => {
                let low = this.data[0x8000 + (tileId * 16) + (j * 2)];
                let high = this.data[0x8000 + (tileId * 16) + (j * 2 + 1)];
                return flatMapRange(8, (k) => {
                    let pl = low >> k & 1;
                    let ph = high >> k & 1;
                    let p = ph << 1 | pl;
                    return COLORS[p];
                });
            }));
            ctx.putImageData(new ImageData(data, 8,8), 0, 0);
        })
    }

    renderTile(n) {
        let x = n % 4;
        let y = Math.floor(n / 4);
        let gridName = `o${y}${x}`;
        let o_addr = 0xFE00 + (n * 4);
        let o_y = this.data[o_addr];
        let o_x = this.data[o_addr + 1];
        let o_t = this.data[o_addr + 2];
        let o_a = this.data[o_addr + 3];
        return html`
            <div class="o-card" style="grid-area: ${gridName};">
                <canvas width="8" height="8" style="width: 100px; height: 100px;"></canvas>
                <div style="display: flex; flex-direction: row;">
                    <span style="width: 50%">X:${o_x}</span>
                    <span style="width: 50%">Y:${o_y}</span>

                </div>
                <div style="display: flex; flex-direction: row;">
                    <span style="width: 50%">T:${o_t}</span>
                    <span style="width: 50%">A:${o_a}</span>
                </div>
            </div>
        `
    }

    render() {
        return html`
            ${Array(40).fill(0).map((_, i) => this.renderTile(i))}
        `
    }

}

customElements.define('oam-debug', OAM)