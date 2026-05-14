cmake_minimum_required(VERSION 3.16)
project(luminaire VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 REQUIRED COMPONENTS Widgets Network)

if(APPLE)
    set_source_files_properties(data/luminaire.icns PROPERTIES
        MACOSX_PACKAGE_LOCATION "Resources"
    )
endif()

add_executable(luminaire
    src/main.cpp
    src/KeyLightAPI.cpp
    src/Config.cpp
    src/MainWindow.cpp
    $<$<BOOL:${APPLE}>:data/luminaire.icns>
)

target_include_directories(luminaire PRIVATE src)
target_link_libraries(luminaire PRIVATE Qt6::Widgets Qt6::Network)

if(APPLE)
    set_target_properties(luminaire PROPERTIES
        MACOSX_BUNDLE TRUE
        MACOSX_BUNDLE_BUNDLE_NAME "Luminaire"
        MACOSX_BUNDLE_GUI_IDENTIFIER "com.luminaire.app"
        MACOSX_BUNDLE_SHORT_VERSION_STRING "${PROJECT_VERSION}"
        MACOSX_BUNDLE_BUNDLE_VERSION "${PROJECT_VERSION}"
        MACOSX_BUNDLE_INFO_PLIST "${CMAKE_SOURCE_DIR}/data/Info.plist.in"
    )
endif()

# Installation
include(GNUInstallDirs)

if(NOT APPLE)
    install(TARGETS luminaire
        RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
    )

    install(FILES data/luminaire.desktop
        DESTINATION ${CMAKE_INSTALL_DATADIR}/applications
    )

    install(FILES data/luminaire.svg
        DESTINATION ${CMAKE_INSTALL_DATADIR}/icons/hicolor/scalable/apps
    )
endif()
