import { css, LitElement } from "lit-element";

class Card extends LitElement {
    static get properties() {
        return {
            title: {type: String, attribute: true},
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
        `
    }

    render() {
        return html`
            <span>${this.title}</span>
            <slot></slot>
        `
    }
}

customElements.define('debug-card', Card);