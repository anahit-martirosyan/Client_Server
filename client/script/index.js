const handleStartClick = () => {
    requestItems()
        .then(items => items.forEach(renderItem))
        .catch(alertError)
}

const handleBuyClick = (itemDivElement, itemId) => {
    buy(itemId)
        .then((item) => {
            updateItemStatus(itemDivElement, item)
            // TODO replace with toast message like in alertError
            alert('Done!');
        })
        .catch(alertError)
}
