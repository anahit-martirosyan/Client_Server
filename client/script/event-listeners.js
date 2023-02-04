const onStartButtonClick = (e) => {
    const element = e.target;
    fadeOutButton(element, 100);
    createRippleElement(e.clientX, e.clientY)
        .then(handleStartClick);
}

const fadeOutButton = (element, duration) => {
    let opacity = 100;
    const interval = setInterval(() => {
        element.style.opacity = opacity + '%';
        opacity--;

        if(opacity === 0) {
            element.parentElement.removeChild(element);
            clearInterval(interval);
        }
    }, duration / 100)
}

const createRippleElement = (x, y) => {
    return new Promise((resolve) => {
        const span = document.createElement('span');
        span.className = 'ripple';
        span.style.top = y;
        span.style.left = x;
        span.style.transition = '1.5s';
        document.body.appendChild(span);

        setTimeout(() => {
            document.body.removeChild(span);
            resolve()
        }, 500)
    })
}

const onByClick = (itemDivElement, itemId) => {
    handleBuyClick(itemDivElement, itemId);
}
