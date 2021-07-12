import { LitElement, html, css } from "lit-element";
import Model from "../model";


class Joypad extends LitElement {
    static get properties() {
        return {data: {attribute: false}};
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
                    "title . . . . ."
                    ". up . . . a"
                    "left . right . b ."
                    ". down . . . ."
                    ". . start select . .";
            }

            :host > p {
                margin-top: 0px;
            }

            .button {
                border-radius: 50%;
                padding: 20px;
                width: 20px;
                height: 20px;
                text-align: center;
                background-color: var(--red);
                color: white;
            }

            .pill {
                border-radius: 10px;
                padding: 10px 0px;
                width: 100px;
                background-color: var(--red);
                color: white;
                text-align:center;
            }

            .pressed {
                background-color: var(--primary);
            }
        `
    }

    isPressed() {
        let action = Model.dmg.get_action_buttons();
        let arrow = Model.dmg.get_arrow_buttons();
        return [
            action & 1,      // 0 a
            action >> 1 & 1, // 1 b 
            action >> 2 & 1, // 2 select
            action >> 3 & 1, // 3 start
            arrow & 1,       // 4 right
            arrow >> 1 & 1,  // 5 left
            arrow >> 2 & 1,  // 6 up
            arrow >> 3 & 1,  // 7 down
        ];
    }

    render() {
        let pressed = this.isPressed();
        return html`
            <p>Joypad</p>
            <div style="grid-area: up;"     class="${pressed[6] ? 'pressed' : ''} button">⬆</div>
            <div style="grid-area: left;"   class="${pressed[5] ? 'pressed' : ''} button">⬅</div>
            <div style="grid-area: right;"  class="${pressed[4] ? 'pressed' : ''} button">➡</div>
            <div style="grid-area: down;"   class="${pressed[7] ? 'pressed' : ''} button">⬇</div>
            <div style="grid-area: a;"      class="${pressed[0] ? 'pressed' : ''} button">A</div>
            <div style="grid-area: b;"      class="${pressed[1] ? 'pressed' : ''} button">B</div>
            <div style="grid-area: start;"  class="${pressed[3] ? 'pressed' : ''} pill">Start</div>
            <div style="grid-area: select;" class="${pressed[2] ? 'pressed' : ''} pill">Select</div>
        `;
    }
}

customElements.define('joypad-debug', Joypad)