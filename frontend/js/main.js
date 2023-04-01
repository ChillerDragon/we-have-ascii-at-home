const viewNum = document.querySelector('#view-value')
const titleDom = document.querySelector('.title')
const commentForm = document.querySelector('#comment-form')
const messageInput = document.querySelector('#message')
const authorInput = document.querySelector('#author')
const commentsBox = document.querySelector('.comments')
const formFlashAlertAuthorContainer = document.querySelector('.form-flash-alert-author-container')
const formFlashAlertMessageContainer = document.querySelector('.form-flash-alert-message-container')
const backendFlashAlertContainer = document.querySelector('.backend-flash-alert-container')

const backendUrl = 'https://api-whaah.zillyhuhn.com'
const params = new URLSearchParams(document.location.search);

const cast = params.get('a') ? params.get('a') : 'twnet'
titleDom.innerHTML = cast

const addBackendError = (err) => {
    const errSlug = err.replace(/[^a-zA-Z]/g, '-')
    const duplicate = backendFlashAlertContainer
        .querySelector(`[data-msg="${errSlug}"]`)
    if (duplicate) {
        return
    }
    backendFlashAlertContainer.innerHTML += `
    <div class="flash-alert" data-msg="${errSlug}">
        ${err}
    </div>
    `
}

const clearBackendErrors = () => {
    backendFlashAlertContainer.innerHTML = ''
}

const addFormInputAuthorError = (err) => {
    const errSlug = err.replace(/[^a-zA-Z]/g, '-')
    const duplicate = formFlashAlertAuthorContainer
        .querySelector(`[data-msg="${errSlug}"]`)
    if (duplicate) {
        return
    }
    formFlashAlertAuthorContainer.innerHTML += `
    <div class="flash-alert" data-msg="${errSlug}">
        ${err}
    </div>
    `
}

const clearFormInputAuthorErrors = () => {
    formFlashAlertAuthorContainer.innerHTML = ''
}

const addFormInputMessageError = (err) => {
    const errSlug = err.replace(/[^a-zA-Z]/g, '-')
    const duplicate = formFlashAlertMessageContainer
        .querySelector(`[data-msg="${errSlug}"]`)
    if (duplicate) {
        return
    }
    formFlashAlertMessageContainer.innerHTML += `
    <div class="flash-alert">
        ${err}
    </div>
    `
}

const clearFormInputMessageErrors = () => {
    formFlashAlertMessageContainer.innerHTML = ''
}

authorInput.addEventListener('keyup', () => {
    // "^[a-zA-Z0-9_-]{1,32}$"
    clearFormInputAuthorErrors()
    const author = authorInput.value
    if (!author || author === "") {
        addFormInputAuthorError('Author can not be empty')
        return
    }
    if (author.length < 1) {
        addFormInputAuthorError('Author has to be at least 1 character long')
    }
    const maxLen = 31
    if (author.length > maxLen) {
        addFormInputAuthorError(`Author too long ${message.length}/${maxLen}`)
    }
    if ("^[a-zA-Z0-9_-]+$".test(author)) {
        addFormInputAuthorError('Author contains illegal characters')
    }
})

messageInput.addEventListener('keyup', () => {
    // "^[a-zA-Z0-9\.,:!?=*#\\()\[\]{}_\n -]+$"
    clearFormInputMessageErrors()
    const message = messageInput.value
    const author = authorInput.value
    if (!author || author === "") {
        addFormInputAuthorError('Author can not be empty')
    }
    if (message.length < 1) {
        addFormInputMessageError('Comment has to be at least 1 character long')
    }
    const maxLen = 2048
    if (message.length >= maxLen) {
        addFormInputMessageError(`Comment too long ${message.length}/${maxLen}`)
    }
})

commentForm.addEventListener('submit', (event) => {
    event.preventDefault()
    const message = messageInput.value
    const author = authorInput.value
    if (!message || message === '') {
        return
    }
    clearBackendErrors()
    fetch(`${backendUrl}/comments/${cast}.cast`, {
        method: 'post',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            author: author,
            message: message,
            timestamp: new Date().toISOString()
        })
    })
        .then(res => res.json())
        .then(data => {
            console.log(data)
            if (data.error) {
                addBackendError(data.error)
            } else {
                getComments()
            }
        })
})

const player = AsciinemaPlayer.create(`/casts/${cast}.cast`, document.getElementById('demo'), {
    fit: 'height'
});

player.addEventListener('play', () => {
    console.log('playing!')
    fetch(`${backendUrl}/view/${cast}.cast`)
})

player.addEventListener('pause', () => {
    console.log("paused!")
})

player.addEventListener('ended', () => {
    console.log("ended!")
})

const getViews = () => {
    fetch(`${backendUrl}/get_views/${cast}.cast`)
        .then(res => res.json())
        .then(data => {
            // console.log(data)
            viewNum.innerHTML = data.views
        })
}

const getComments = () => {
    fetch(`${backendUrl}/comments/${cast}.cast`)
        .then(res => res.json())
        .then(comments => {
            commentsBox.innerHTML = ''
            comments.forEach((comment) => {
                const commentHtml = `
                <div class="comment">
                    <div class="comment-header">
                        <div class="comment-author">${comment.author}</div>
                        <div class="comment-timestamp">${comment.timestamp.split('.')[0]}</div>
                    </div>
                    <pre class="code-snippet comment-message">${comment.message}</pre>
                </div>
                `
                commentsBox.insertAdjacentHTML('beforeend', commentHtml)
            })
        })
}

getViews()
getComments()

setInterval(getViews, 2000)
setInterval(getComments, 2000)
