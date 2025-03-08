document.addEventListener("DOMContentLoaded", () => {
    const fileInput = document.getElementById("file-input");
    const fileList = document.getElementById("file-list");
    const uploadButton = document.getElementById("upload-button");
    const uploadForm = document.getElementById("upload-form");

    let filesArray = [];

    // Event listener to handle file selection
    fileInput.addEventListener("change", (event) => {
        // Add selected files to the array
        for (const file of event.target.files) {
            filesArray.push(file);
        }

        // Update the displayed file list
        updateFileList();

        // Enable the upload button if there are files in the array
        uploadButton.disabled = filesArray.length === 0;

        // Clear the file input value to allow re-selecting the same files
        fileInput.value = "";
    });

    // Function to update the visual file list
    function updateFileList() {
        // Clear the current list in the HTML
        fileList.innerHTML = "";

        // Add each file to the visual list
        filesArray.forEach((file, index) => {
            const li = document.createElement("li");
            li.textContent = `${file.name} (${(file.size / 1024).toFixed(2)} KB)`;

            // Add a remove button for each file
            const removeButton = document.createElement("button");
            removeButton.textContent = "Remove";
            removeButton.style.marginLeft = "10px";
            removeButton.addEventListener("click", () => {
                // Remove the file from the array
                filesArray.splice(index, 1);
                updateFileList();
                // Disable the upload button if no files remain
                uploadButton.disabled = filesArray.length === 0;
            });

            li.appendChild(removeButton);
            fileList.appendChild(li);
        });
    }

    // Handle form submission and send files using FormData
    uploadForm.addEventListener("submit", (event) => {
        event.preventDefault(); // Prevent default form submission
        const formData = new FormData();

        // Append each file to FormData
        filesArray.forEach(file => {
            formData.append("files", file);
        });

        // Use Fetch API to send the form data
        fetch("/", {
            method: "POST", body: formData
        })
            .then(response => {
                if (response.ok) {
                    alert("Files uploaded successfully!");
                    // Clear the file array and update the list
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
});
