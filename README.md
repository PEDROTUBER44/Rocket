# Secure Cloud Storage

This project is a secure cloud storage application built with Rust, Axum, and PostgreSQL. It provides end-to-end encryption for all files, ensuring that only the user can access their data.

## Features

- **End-to-End Encryption:** All files are encrypted on the client-side before being uploaded to the server.
- **Secure Authentication:** User authentication is handled with Argon2, a secure password hashing algorithm.
- **File and Folder Management:** Users can create, delete, and list files and folders.
- **Chunked Uploads:** Large files are split into smaller chunks for more reliable uploads.
- **Rate Limiting:** The application includes rate limiting to prevent abuse.
- **Secure Cookies:** Session and CSRF tokens are stored in secure, HTTP-only cookies.

## Getting Started

### Prerequisites

- Rust 1.60 or higher
- PostgreSQL 13 or higher
- Redis 6 or higher

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/secure-cloud-storage.git
   ```
2. Set up the environment variables:
   ```bash
   cp .env.example .env
   ```
   Update the `.env` file with your database and Redis connection details, and generate a master key:
   ```bash
   openssl rand -hex 32
   ```
3. Run the database migrations:
   ```bash
   sqlx migrate run
   ```
4. Start the application:
   ```bash
   cargo run
   ```

## API Endpoints

The following are the available API endpoints:

- `POST /api/auth/register`: Register a new user.
- `POST /api/auth/login`: Log in a user.
- `POST /api/auth/logout`: Log out a user.
- `POST /api/auth/change-password`: Change a user's password.
- `GET /api/files`: List all files for the current user.
- `POST /api/files/upload/init`: Initialize a file upload.
- `POST /api/files/upload/chunk`: Upload a chunk of a file.
- `POST /api/files/upload/finalize`: Finalize a file upload.
- `POST /api/files/upload/cancel`: Cancel a file upload.
- `GET /api/files/{file_id}`: Download a file.
- `DELETE /api/files/{file_id}`: Delete a file.
- `GET /api/folders`: List all folders for the current user.
- `POST /api/folders`: Create a new folder.
- `GET /api/folders/{folder_id}`: Get a folder's statistics.
- `DELETE /api/folders/{folder_id}`: Delete a folder.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request if you have any improvements.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.
