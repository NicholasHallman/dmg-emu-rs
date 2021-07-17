import { LitElement, html, css } from "lit-element";

class Tabbed extends LitElement {

    static get styles() {
        return css`
            :host {
                background-color: #1D1E23;
                width: calc(100% - 10px);
                padding: 5px;
                border-radius: 5px;
                display: block;
                margin-top: 10px;
            }

            .tabs {
                display: flex;
                flex-direction: row;
                margin: 0px 5px;
                margin-left: -5px;
                margin-top: -5px;
            }

            .tab {
                border: solid 1px var(--red);
                background-color: rgb(29, 30, 35);
                border-bottom: none;
                padding: 0px 20px;
                cursor: pointer;
                user-select: none;
            }

            .active {
                background-color: var(--red);
            }
        `       
    }

    static get properties() {
        return {
            titles: { attribute: false},
            elements: { attribute: false},
            active: { attribute: false }
        }
    }

    connectedCallback() {
        super.connectedCallback();
        this.active = 0;
    }

    _handleChange(i) {
        this.active = i;
    } 

    render() {
        return html`
            <div class="tabs">
                ${this.titles.map((title, i) => html`<span class="tab ${this.active === i ? 'active' : ''}" @click=${() => this._handleChange(i)} @keypress=${() => this._handleChange(i)} tabindex="$1" class="tab">${title}<span>`)}
            </div>
            ${this.elements[this.active]}
        `
    }
}

customElements.define('tabbed-card', Tabbed);