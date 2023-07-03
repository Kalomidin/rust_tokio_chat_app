# Chat Room Owner Functionality

This project implements chat room owner functionality, allowing the owner to perform various actions within the chat room.

## Features

- **Owner Privileges**: The chat room owner has special privileges and permissions within the room.
    - **Kick Out Users**: The owner can kick out other users from the chat room.
    - **TODO**: Other features on its way
- **Room Deletion**: When all users leave the room, it is automatically marked as deleted.
- **Notification on Kick Out**: When a user gets kicked out by the owner, they receive a notification about the event.
- **Messaging**: Users can send messages to each other within the chat room.

## Requirements

- rust
- postgres
- docker

## Application Installation Steps

1. Clone the repository:
2. Navigate to the project directory and start the Docker containers:
    ```bash
    docker-compose up -d
    ```
3. Build and run the application using Cargo:
    ```bash
    cargo run
    ```
4. Open the following Postman link to access the API documentation and test the endpoints [link](https://app.getpostman.com/join-team?invite_code=bfa2daa5a7cbadad1f29c50e8252ed1a&target_code=43cf5096b948cf03c2f1e73e40cd22c8)
5. In Postman, you can create a user and a room to start enjoying the chatting experience.

Please note that you may need to adjust the steps based on your specific project setup or any additional requirements.