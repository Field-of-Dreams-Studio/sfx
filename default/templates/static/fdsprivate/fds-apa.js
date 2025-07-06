function inform(message) {
    var errorDiv = document.createElement('div');
    errorDiv.className = 'alert alert-danger alert-dismissible fade show';
    errorDiv.setAttribute('role', 'alert');
    errorDiv.style.position = 'fixed';
    errorDiv.style.top = '20px';
    errorDiv.style.right = '-50%';
    errorDiv.style.width = 'auto';
    errorDiv.style.zIndex = '9999';
    errorDiv.textContent = message;

    // Append the close button if needed
    // var closeButton = document.createElement('button');
    // closeButton.setAttribute('type', 'button');
    // closeButton.className = 'close';
    // closeButton.setAttribute('data-dismiss', 'alert');
    // closeButton.setAttribute('aria-label', 'Close');
    // closeButton.innerHTML = '<span aria-hidden="true">&times;</span>';
    // errorDiv.appendChild(closeButton);

    document.body.appendChild(errorDiv);

    // Animate right position to 20px
    setTimeout(function() {
        errorDiv.style.transition = 'right 0.5s';
        errorDiv.style.right = '20px';
    }, 10);

    // Remove the alert after 5 seconds
    setTimeout(function() {
        errorDiv.style.transition = 'right 0.5s';
        errorDiv.style.right = '-50%';
        setTimeout(function() {
            document.body.removeChild(errorDiv);
        }, 500);
    }, 5000);
}
