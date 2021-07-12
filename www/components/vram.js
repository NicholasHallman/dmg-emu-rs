import { LitElement, html, css } from "lit-element";
import Model from '../model';

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

class VRAM extends LitElement {
    static get properties() {
        return {
            data: {attribute: false},
            play: {attribute: false}
        };
    }

    static get styles() {
        return css`
            :host {
                display: block;
                background-color: #1D1E23;
                width: calc(100% - 10px);
                padding: 5px;
                border-radius: 5px;
                margin-top: 10px;
            }

            :host > p {
                margin-top: 0px;
            }
            pre {
                overflow-y: scroll;
                height: 300px;
                font-size: 14px;
            }
            canvas {
                width: 100%;
                image-rendering: -moz-crisp-edges;
                image-rendering: -webkit-crisp-edges;
                image-rendering: pixelated;
                image-rendering: crisp-edges;
            }
        `
    }

    constructor() {
        super();
        if (!this.hasAttribute('tabindex')) {
            this.setAttribute('tabindex', 0);
        }
    }

    updated() {
        if (this.play) return;
        let ctx = this.shadowRoot.querySelector('canvas').getContext('2d');
        let tiles = [];
        let start = 0x8000;
        let end = 0x9FFF;
        let cur = 0;
        while (cur + start <= end) {
            let i = Math.floor(cur / 16);
            let row = (cur/2) % 8;
            if (row === 0) {
                tiles[i] = new Uint8ClampedArray(256); // 8 rows * 8 columns * 4 colors
            }
            // get the two bytes that make up a line
            let bh = this.data[start + cur];
            let bl = this.data[start + cur + 1];
            // get the colors for the line
            let rowcolors = Array(8).fill(0).flatMap((_, i) => {
                let ph = bh >> (7 - i) & 1;
                let pl = bl >> (7 - i) & 1;
                let n = pl << 1 | ph;
                return COLORS[n]
            });
            tiles[i].set(rowcolors, row * rowcolors.length);
            // convert the index to an x,y on the canvas
            // draw
            cur += 2;
        }
        tiles.forEach((tile, i) =>{
            let x = (i % 24) * 8;
            let y = Math.floor(i / 24) * 8;
            ctx.putImageData(new ImageData(tile, 8, 8), x, y);
        })
    }

    render() {
        return html`
            <p>VRAM</p>
            <canvas width="192" height="128"></canvas>
        `
    }
}

customElements.define('vram-debug', VRAM);