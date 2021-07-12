import { LitElement, html, css} from "lit-element";


class Memory extends LitElement {
    static get properties() {
        return {
            data: {type: Object, attribute: false},
            play: {type: Boolean, attribute: false},
            viewOverlay: {attribute: false}
        }
    }

    static get styles() {
        return css`
            :host {
                display: block;
                position: relative;
                background-color: #1D1E23;
                width: calc(100% - 10px);
                padding: 5px;
                height: 337px;
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
                line-height: 16px;
            }

            .memory::-webkit-scrollbar {
                width: 10px;               /* width of the entire scrollbar */
            }

            .memory::-webkit-scrollbar-track {
                background: rgb(37, 39, 49);        /* color of the tracking area */
            }

            .memory::-webkit-scrollbar-thumb {
                background-color: var(--white);    /* color of the scroll thumb */
                border-radius: 20px;       /* roundness of the scroll thumb */
            }
            .overlay{
                position:absolute;
                color: white;
                width: calc(100% - 5px);
                margin-left: -5px;
                display: none;
                bottom: 0px;
            }
            .overlay.draw {
                display: block;
            }
            .hidden {
                color: white;
                border: none;
                width: 100%;
                background-color: black;
            }
        `
    }

    constructor() {
        super();
        if (!this.hasAttribute('tabindex')) {
            this.setAttribute('tabindex', 0);
        }
        this.addEventListener('keypress', (e) => {
            if (e.key === ':') {
                this.viewOverlay = true;
            }
            if (e.key === "Enter" && this.viewOverlay) {
                let line = this.shadowRoot.querySelector('.overlay>input').value;
                this.shadowRoot.querySelector('.overlay>input').value = "";
                line = line.substring(1);
                let n = (Number.parseInt(line, 16) / 16);
                this.shadowRoot.querySelector('pre').scroll(0, n * 16);
                this.viewOverlay = false;
            }
        })
        this.viewOverlay = false;
    }

    hex(n) {
        let s = n.toString(16).toUpperCase();
        return s.length === 1 ? `00${s}0`
             : s.length === 2 ? `0${s}0`
             : s.length === 3 ? `${s}0`
             : s
    }

    hex2(n) {
        if (!n) return '00';
        let s = n.toString(16).toUpperCase();
        return s.length === 1 ? `0${s}`
             : s;
    }

    dataAt(i) {
        return `${this.hex2(this.data[i * 16])} ${this.hex2(this.data[i * 16 + 1])} ${this.hex2(this.data[i * 16 + 2])} ${this.hex2(this.data[i * 16 + 3])} ${this.hex2(this.data[i * 16 + 4])} ${this.hex2(this.data[i * 16 + 5])} ${this.hex2(this.data[i * 16 + 6])} ${this.hex2(this.data[i * 16 + 7])} ${this.hex2(this.data[i * 16 + 8])} ${this.hex2(this.data[i * 16 + 9])} ${this.hex2(this.data[i * 16 + 10])} ${this.hex2(this.data[i * 16 + 11])} ${this.hex2(this.data[i * 16 + 12])} ${this.hex2(this.data[i * 16 + 13])} ${this.hex2(this.data[i * 16 + 14])} ${this.hex2(this.data[i * 16 + 15])}`
    }

    updated() {
        if (this.viewOverlay) {
            this.shadowRoot.querySelector('.overlay>input').focus();
        }
    }

    renderShortcut() {
        return html`
            <div class="overlay ${this.viewOverlay ? 'draw' : ''}">
            <input @blur=${() => {
                this.viewOverlay = false;
                this.shadowRoot.querySelector('.overlay>input').value="";
                }} class="hidden"></input> 
            </div>
        `
    }

    render() {
        if (this.play || this.data.length === 0) {
            return html`<p>Memory</p><br>${this.renderShortcut()}`
        }
        let n = 65536 / 16;
        let s = Array(n)
        .fill(0)
        .map(
            (_, i) => `${this.hex(i)}: ${this.dataAt(i)}`
            
        ).join('\n');
    
        return html`
            <p>Memory</p>
            <pre class="memory">${s}</pre>
            ${this.renderShortcut()}
        `;
    }
}

customElements.define('memory-debug', Memory);

