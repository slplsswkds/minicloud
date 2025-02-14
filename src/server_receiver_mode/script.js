const fileInput = document.getElementById('file-input');
const fileList = document.getElementById('file-list');
const uploadButton = document.getElementById('upload-button');

let filesArray = [];

// Отримуємо максимальний розмір файлу з сервера (можна змінити на статичне значення)
fetch("/max_size")
    .then(response => response.text())
    .then(size => document.getElementById("max-size").textContent = size)
    .catch(() => document.getElementById("max-size").textContent = "Unknown");

// Обробник вибору файлів
fileInput.addEventListener('change', (event) => {
    for (const file of event.target.files) {
        filesArray.push(file);
    }

    updateFileList();
    uploadButton.disabled = filesArray.length === 0;
    fileInput.value = "";
});

// Оновлення списку файлів
function updateFileList() {
    fileList.innerHTML = "";

    filesArray.forEach((file, index) => {
        const li = document.createElement('li');
        li.textContent = `${file.name} (${(file.size / 1024).toFixed(2)} KB)`;

        const removeButton = document.createElement('button');
        removeButton.textContent = "Remove";
        removeButton.addEventListener('click', () => {
            filesArray.splice(index, 1);
            updateFileList();
            uploadButton.disabled = filesArray.length === 0;
        });

        li.appendChild(removeButton);
        fileList.appendChild(li);
    });
}

// Обробка завантаження файлів
document.getElementById('upload-form').addEventListener('submit', (event) => {
    event.preventDefault();
    const formData = new FormData();

    filesArray.forEach(file => {
        formData.append('files', file);
    });

    fetch("/", {
        method: "POST",
        body: formData
    })
        .then(response => {
            if (response.ok) {
                alert("Files uploaded successfully!");
                filesArray = [];
                updateFileList();
                uploadButton.disabled = true;
            } else {
                alert("Failed to upload files.");
            }
        })
        .catch(error => {
            console.error("Error:", error);
            alert("An error occurred while uploading files.");
        });
});
