const sections = {};

const getSection = (category) => {
    if(sections[category]) {
        return sections[category];
    }

    sections[category] = document.createElement('section');
    sections[category].id = category;
    sections[category].className = 'items-section';
    sections[category].innerHTML = `<h2 class="section-title">${category}</h2>`;
    document.querySelector('#items-wrapper').appendChild(sections[category]);

    return sections[category];
}

const renderItem = (item) => {
    const a = document.createElement('a');
    const section = getSection(item.category);

    a.appendChild(createItemImage(item.image, item.name));
    a.appendChild(createItemDescription(item));
    a.animate([
        { transform: 'translateX(100%)', opacity: '0'},
        { transform: 'translateX(0)', opacity: '1'}
    ], {duration: 300, delay: section.children.length * 300, fill: 'backwards', easing: 'ease-out'})
    a.className = 'item';
    section.appendChild(a);
    section.querySelector('.section-title').textContent = `${item.category} (${section.children.length - 1})`;
}

const createItemImage = (src, alt) => {
    const image = document.createElement('img');
    image.alt = alt;
    image.src = src;
    image.className = 'item-image';
    return image;
}

const createItemDescription = (item) => {
    const div = document.createElement('div');
    const pName = `<p class="item-name">${item.name}</p>`
    const pPrice = `<p class="item-price">$${item.price}</p>`
    const pStatus = getStatusSpan(item.status);
    const button = document.createElement('button');
    button.className = 'purchase-btn';
    button.textContent = 'BUY';
    button.onclick = () => onByClick(div, item.id);
    div.className = 'item-description';
    div.innerHTML += (pName + pPrice);
    div.appendChild(pStatus);
    div.appendChild(button);
    return div;
}

const getStatusSpan = (status) => {
    const p = document.createElement('p');
    p.classList.add('item-status');
    p.classList.add(status ? 'status-available' : 'status-unavailable');
    p.innerText = status ? 'Status: available' : 'Status: unavailable';
    return p;
}

const updateItemStatus = (divElement, item) => {
    const pStatus = divElement.querySelector('.item-status');
    pStatus.innerText = item.status ? 'Status: available' : 'Status: unavailable';
    if(item.status) {
        pStatus.classList.add('status-available');
        pStatus.classList.remove('status-unavailable');
    } else {
        divElement.querySelector('button').remove();
        pStatus.classList.remove('status-available');
        pStatus.classList.add('status-unavailable');
    }
}

const alertError = (error) => {
    const errorPanel = document.querySelector('#error-panel');
    errorPanel.innerText = 'Something went wrong!';
    errorPanel.style.display = 'block';
    errorPanel.classList.add('error-animation');
    setTimeout(() => {
        errorPanel.classList.remove('error-animation');
        errorPanel.style.display = 'none';
    }, 15000);
    console.error(error);
}
