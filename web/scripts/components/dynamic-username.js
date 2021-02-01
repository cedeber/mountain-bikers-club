import { LitElement, html, css } from 'https://cdn.skypack.dev/lit-element?min'

export class DynamicUsername extends LitElement {
  static get styles () {
    // language=css
    return css`
        .text-primary {
            color: var(--bs-blue)
        }

        .text-danger {
            color: var(--bs-red)
        }

        .text-success {
            color: var(--bs-green)
        }
    `
  }

  static get properties () {
    return {
      from: { type: String },
      username: { type: String },
      isWrong: { type: Boolean, attribute: false },
    }
  }

  _handleChange (event) {
    let value = event.target.value
    let isCorrect = /^[a-zA-Z0-9_]{4,32}$/.test(value)
    this.isWrong = !isCorrect
    this.username = value
    this.requestUpdate()
  }

  constructor () {
    super()
    this._handleChange = this._handleChange.bind(this)
    this.isWrong = false
  }

  connectedCallback () {
    super.connectedCallback()
    this.usernameInput = document.getElementById(this.from)
    if (this.usernameInput) {
      this.usernameInput.addEventListener('input', this._handleChange)
      this.usernameInput.addEventListener('change', this._handleChange)
    }
  }

  disconnectedCallback () {
    if (this.usernameInput) {
      this.usernameInput.removeEventListener('input', this._handleChange)
      this.usernameInput.removeEventListener('change', this._handleChange)
    }
    super.disconnectedCallback()
  }

  render () {
    return html`
      <span class="text-primary">mountainbikers.club/<strong
          class="${this.isWrong ? 'text-danger' : 'text-success'}">${this.username}</strong></span>
    `
  }
}