import ClipboardJS from 'https://cdn.skypack.dev/clipboard?min'
import autosize from 'https://cdn.skypack.dev/autosize?min'
import { AddressMap } from './components/address-map.js'
import { CurrentTimezone } from './components/current-timezone.js'
import { DateTime } from './components/date-time.js'
import { DynamicUsername } from './components/dynamic-username.js'

// Custom Elements
customElements.define('address-map', AddressMap)
customElements.define('current-timezone', CurrentTimezone)
customElements.define('date-time', DateTime)
customElements.define('dynamic-username', DynamicUsername)

// copy to clipboard buttons
new ClipboardJS('.btn-clipboard')

// All Bootstrap Toasts
for (const toast of document.querySelectorAll('.toast')) {
  new bootstrap.Toast(toast).show()
}

// All Bootstrap Tooltips
for (const tooltip of document.querySelectorAll('[data-bs-toggle="tooltip"]')) {
  new bootstrap.Tooltip(tooltip)
}

// Password + eye
for (const passwordEyeGroup of document.querySelectorAll('.password-eye-group')) {
  const button = passwordEyeGroup.querySelector('button')
  const input = passwordEyeGroup.querySelector('input')
  const icon = passwordEyeGroup.querySelector('.eye')

  button.addEventListener('click', () => {
    const currentType = input.getAttribute('type')

    if (currentType === 'password') {
      input.setAttribute('type', 'text')
      icon.classList.remove('fa-eye')
      icon.classList.add('fa-eye-slash')
    } else {
      input.setAttribute('type', 'password')
      icon.classList.remove('fa-eye-slash')
      icon.classList.add('fa-eye')
    }
  })
}

// Autosize Textarea
for (const textarea of document.querySelectorAll('.autosize')) {
  autosize(textarea)
  textarea.addEventListener('focus', () => autosize.update(textarea))
}
