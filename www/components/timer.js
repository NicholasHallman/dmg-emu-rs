import { LitElement, html, css } from "lit-element";

class Timer extends LitElement {
    static get properties() {
        return {
            timerIO: {attribute: false}
        }
    }

    static get styles() {
        return css`
            :host {
                width: calc(100% - 10px);
                padding: 10px;
            }
            .row {
                margin-bottom: 10px;
            }
        `;
    }

    get tac() {
        let mode = this.timerIO.TAC & 3;
        let enabled = this.timerIO.TAC & 4;
        let speed = mode === 0 ? 4096
            : mode === 1 ? 262144
            : mode === 2 ? 65536
            : 16384;
        return `${speed} | ${enabled ? 'ON' : 'OFF'}`;
    }

    render() {
        return html`
            <div class="row">
                <span>DIV: ${this.timerIO.DIV}</span>
                <span>TIMA: ${this.timerIO.TIMA}</span>
                <span>TMA: ${this.timerIO.TMA}</span>
            </div>
            <span>TAC: ${this.tac}</span>
        `
    }
}

customElements.define('timer-debug', Timer);