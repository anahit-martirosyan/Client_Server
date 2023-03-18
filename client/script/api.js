// const itemList = [{"category":"ski equipment","id":1,"image":"https://cdn.shopify.com/s/files/1/0055/1809/8522/products/190310_OliveOil_CarsonMeyer_Lifestyle-63_7af032de-1b8c-468c-80cb-4021dee4c67f.jpg?v=1653070224","name":"","price":200,"status":"available"},{"category":"ski equipment","id":2,"image":"https://cdn.shopify.com/s/files/1/0055/1809/8522/products/190310_OliveOil_CarsonMeyer_Lifestyle-63_7af032de-1b8c-468c-80cb-4021dee4c67f.jpg?v=1653070224","name":"","price":400,"status":"available"},{"category":"ski wear","id":1,"image":"https://cdn.shopify.com/s/files/1/0055/1809/8522/products/190310_OliveOil_CarsonMeyer_Lifestyle-63_7af032de-1b8c-468c-80cb-4021dee4c67f.jpg?v=1653070224","name":"","price":20,"status":"available"},{"category":"ski wear","id":2,"image":"https://cdn.shopify.com/s/files/1/0055/1809/8522/products/190310_OliveOil_CarsonMeyer_Lifestyle-63_7af032de-1b8c-468c-80cb-4021dee4c67f.jpg?v=1653070224","name":"","price":10,"status":"available"}];
const baseURL = 'http://127.0.0.1:5100';
const requestItems = () => {
    // return Promise.resolve(itemList);
    return fetch(baseURL + '/items')
        .then(response => response.json())
}

const buy = (itemId) => {
    return new Promise((resolve, reject) => {
        const xhr = new XMLHttpRequest();
        xhr.open("PUT", baseURL + '/purchase?id=' + itemId, true);
        xhr.setRequestHeader('Content-type', 'application/json');

        // function execute after request is successful
        xhr.onreadystatechange = function (e) {
            if (this.status === 200) {
                e.srcElement.response && resolve(JSON.parse(e.srcElement.response))
            } else {
                reject(e);
            }
        }
        // Sending our request
        xhr.send();
    })

}
