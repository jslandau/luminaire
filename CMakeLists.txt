cmake_minimum_required(VERSION 3.16)
project(luminaire VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 REQUIRED COMPONENTS Widgets Network)

add_executable(luminaire
    src/main.cpp
    src/KeyLightAPI.cpp
    src/Config.cpp
    src/MainWindow.cpp
)

target_include_directories(luminaire PRIVATE src)
target_link_libraries(luminaire PRIVATE Qt6::Widgets Qt6::Network)

# Installation
include(GNUInstallDirs)

install(TARGETS luminaire
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)

install(FILES data/luminaire.desktop
    DESTINATION ${CMAKE_INSTALL_DATADIR}/applications
)

install(FILES data/luminaire.svg
    DESTINATION ${CMAKE_INSTALL_DATADIR}/icons/hicolor/scalable/apps
)
