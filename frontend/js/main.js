const viewNum = document.querySelector('#view-value')
const titleDom = document.querySelector('.title')
const commentForm = document.querySelector('#comment-form')
const commentInput = document.querySelector('#comment')
const commentsBox = document.querySelector('.comments')

const backendUrl = 'https://api-whaah.zillyhuhn.com'
const params = new URLSearchParams(document.location.search);

const cast = params.get('a') ? params.get('a') : 'twnet'
titleDom.innerHTML = cast

commentForm.addEventListener('submit', (event) => {
    event.preventDefault()
    const msg = commentInput.value
    if (!msg || msg === '') {
        return
    }
    fetch(`${backendUrl}/comments/${cast}.cast`, {
        method: 'post',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({author: 'nameless', message: msg})
    })
        .then(res => res.json())
        .then(data => {
            console.log(data)
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
            console.log(data)
            viewNum.innerHTML = data.views
        })
}

const getComments = () => {
    fetch(`${backendUrl}/comments/${cast}.cast`)
        .then(res => res.json())
        .then(data => {
            commentsBox.innerHTML = ''
            data.comments.forEach((comment) => {
                const commentHtml = `
                <div class="comment">
                    ${comment.message}
                </div>
                `
                commentsBox.insertAdjacentElement('beforeend', commentHtml)
            })
        })
}

getViews()
getComments()

setInterval(getViews, 2000)
setInterval(getComments, 2000)
