import { LitElement, css, html } from "lit-element";

class Serial extends LitElement {
    static get properties() {
        return {
            buffer: {attribute: false}
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
            height: 200px;
            border-radius: 5px;
            margin-top: 10px;
        }
        
        :host > p {
            margin-top: 0px;
        }
        `
    }

    constructor() {
        super();
        this.buffer = "";
    }

    render() {
        return html`
            <p>Serial</p>
            <pre>${this.buffer}</pre>
        `
    }
}

customElements.define('serial-debug', Serial);