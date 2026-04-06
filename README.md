> [!WARNING]
> I am currently rebuilding Rustmius entirely in native Rust using GTK. This version will initially support only Linux, and will later be ported to Windows and macOS. This change in the tech stack will be the final one, allowing me to focus on features and later refine the UI.

<div align="center">
    <table>
        <tr>
            <td><img src="rustmius.png" alt="Rustmius Logo" width="100"/></td>
            <td><h1>Rustmius</h1></td>
        </tr>
    </table>
    <p>
        <img src="https://img.shields.io/aur/version/rustmius?label=AUR%20Rustmius&logo=arch-linux&logoColor=white&labelColor=1793d1" alt="AUR Version"/>
        <img src="https://img.shields.io/aur/version/rustmius-bin?label=AUR%20Rustmius%20Bin&logo=arch-linux&logoColor=white&labelColor=1793d1" alt="AUR Version Bin"/>
        <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"/>
        <img src="https://img.shields.io/badge/rust-1.70+-orange.svg" alt="Rust"/>
        <img src="https://img.shields.io/badge/vue-3.5+-green.svg" alt="Vue"/>
    </p>
</div>

## 🎯 Purpose

**Rustmius** is a full local Termius alternative for Linux, built with Tauri (Rust + Vue.js) that simplifies SSH server and key management. Say goodbye to manually editing SSH config files and managing keys through the command line!

### ✨ Key Features

- 🖥️ **Visual Server Management**: Add, edit, and organize your SSH servers with a modern, intuitive interface
- 📁 **Hierarchical Organization**: Organize your servers in folders for better structure
- 🔑 **SSH Key Management**: View and manage your SSH key pairs effortlessly
- 🐳 **Docker Integration**: Manage your Docker containers and images directly from the application
- 📊 **System Monitoring**: Monitor your servers' performance in real-time
- 🔍 **Smart Search**: Quickly find your servers and keys with intelligent filtering
- 🚀 **One-Click Connection**: Connect to servers directly from the app
- 🎨 **Modern UI**: User interface built with Vue.js and Tailwind CSS
- ⚡ **Optimal Performance**: Rust backend for optimal performance and memory safety
- 🔧 **Multi-Terminal Support**: Works with multiple terminals (foot, gnome-terminal, konsole, alacritty, kitty, etc.)

## 🏗️ Architecture

Rustmius is a **Tauri** application that combines:

- **Rust Backend**: System operations management, data storage, and system integration
- **Vue.js 3 Frontend**: Reactive user interface with TypeScript
- **Pinia**: Centralized state management
- **Vue Router**: Navigation between pages
- **Tailwind CSS**: Modern and responsive styling
- **UI Components**: Reusable component library (shadcn/ui style)

### 📂 Project Structure

```
rustmius/
├── src/                    # Frontend source code (Vue.js)
│   ├── pages/             # Application pages
│   │   ├── home.vue       # Home page (server list)
│   │   ├── keys.vue       # SSH key management
│   │   ├── server/        # Server-related pages
│   │   │   ├── index.vue  # Server overview
│   │   │   ├── docker.vue # Docker management
│   │   │   └── monitor.vue # System monitoring
│   │   └── settings.vue   # Application settings
│   ├── components/        # Reusable Vue components
│   │   ├── AppSidebar.vue # Navigation sidebar
│   │   ├── ServerCard.vue # Server card
│   │   ├── DockerCard.vue  # Docker card
│   │   └── ui/            # Base UI components
│   ├── stores/            # Pinia stores
│   │   ├── servers.ts     # Server management
│   │   ├── keys.ts        # SSH key management
│   │   └── settings.ts    # Settings
│   ├── class/             # TypeScript classes
│   │   ├── Server.ts      # Server class
│   │   └── Class.ts       # Utility classes
│   ├── types/             # TypeScript definitions
│   ├── router/             # Vue Router configuration
│   └── lib/               # Utilities
├── src-tauri/             # Backend source code (Rust)
│   ├── src/
│   │   ├── main.rs        # Tauri entry point
│   │   └── lib.rs         # Rust library
│   └── Cargo.toml         # Rust dependencies
└── dist/                   # Production build
```

## 🚀 Installation

### From AUR (Arch Linux)

```bash
# Install from source
yay -S rustmius

# Or install pre-built binary
yay -S rustmius-bin
```

### From Source

#### Prerequisites

- **Rust** (1.70+) : [rustup.rs](https://rustup.rs/)
- **Node.js** (18+) or **Bun** : [nodejs.org](https://nodejs.org/) or [bun.sh](https://bun.sh/)
- **System Dependencies** :
  - Linux : `libwebkit2gtk-4.1-dev`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`

#### Installation

```bash
# Clone the repository
git clone https://github.com/Cleboost/Rustmius.git
cd Rustmius

# Install frontend dependencies
npm install
# or
bun install

# Run in development mode
npm run tauri dev
# or
bun run tauri dev

# Build for production
npm run tauri build
# or
bun run tauri build
```

## 🎮 Usage

### Navigation

The application is organized into several sections accessible via the sidebar:

1. **🏠 Home**: Overview of all your servers organized in folders
2. **🔑 SSH Keys**: Management of your SSH key pairs
3. **⚙️ Settings**: Application configuration

### Server Management

- **Add a server**: Click "New Server" to add an SSH server with a user-friendly interface
- **Organize in folders**: Create folders to organize your servers by project or environment
- **Edit a server**: Click on a server to access its details and options
- **Delete a server**: Use the delete option in the server details

### Docker Features

For each server, you can access:

- **Docker Overview**: List of containers and images
- **Image Management**: View and manage your Docker images
- **Container Management**: Create, start, stop, and delete containers
- **Container Details**: Inspect logs, statistics, and container configuration

### System Monitoring

Monitor your servers' performance with:

- **CPU and memory usage**
- **Network statistics**
- **Disk usage**
- **Real-time graphs**

### SSH Key Management

- **Visualization**: View all your public and private SSH keys
- **Generation**: Create new SSH key pairs
- **Association**: Link keys to specific servers

## 🛠️ Development

### Available Scripts

```bash
# Development
npm run dev          # Start Vite in development mode
npm run tauri dev    # Start Tauri application in development mode

# Build
npm run build        # Build the frontend
npm run tauri build  # Build the complete application

# Code Quality
npm run lint         # Check TypeScript types
npm run knip         # Detect unused code
```

### Testing

```bash
# Run tests
npm test

# Tests with coverage
npm run test:coverage
```

### Data Structure

Servers are stored in a hierarchical structure allowing folder organization:

```typescript
type ServerConfig = Array<Folder | Server>;

interface Folder {
  id: string;
  name: string;
  contents: ServerConfig;
}

interface Server {
  id: string;
  name: string;
  host: string;
  port: number;
  user: string;
  // ... other properties
}
```

## 🤝 Contributing

We welcome contributions from the community! 🎉 Whether you're fixing bugs, adding features, or improving documentation, your help makes this project better for everyone.

### 🛠️ How to Contribute

1. **Fork the repository** 🍴
2. **Create a feature branch** `git checkout -b feature/amazing-feature`
3. **Make your changes** ✨
4. **Test thoroughly** 🧪
5. **Commit with clear messages** 📝 (follow commit conventions)
6. **Push to your fork** 🚀
7. **Open a Pull Request** 🔄

### 🎯 Areas Where We Need Help

- 🐛 **Bug Reports**: Found a bug? Let us know!
- 💡 **Feature Requests**: Have ideas? We'd love to hear them!
- 📚 **Documentation**: Help improve our docs and guides
- 🎨 **UI/UX**: Design improvements and user experience enhancements
- 🧪 **Testing**: Help us test on different systems and configurations
- 🌍 **Translations**: Help us reach users in different languages

### 📋 Pull Request Guidelines

- **Clear descriptions**: Explain what your PR does and why
- **Focused changes**: Keep PRs focused on a single feature or fix
- **Test your changes**: Make sure everything works as expected
- **Follow code style**: Use `cargo fmt` and `cargo clippy` for Rust, `npm run lint` for TypeScript

### 🏆 Recognition

Contributors will be recognized in our README and release notes. We appreciate every contribution, no matter how small! 🙏

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Tauri team** for the desktop application framework
- **Vue.js community** for the excellent frontend ecosystem
- **Rust community** for the backend ecosystem
- **All contributors** who help improve this project
- **Arch Linux community** for the AUR packages

---

<div align="center">
  <p>Made with ❤️ by <a href="https://github.com/Cleboost">Cleboost</a></p>
  <p>⭐ Star this repo if you find it useful!</p>
</div>
