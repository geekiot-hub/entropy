cmake_minimum_required(VERSION 3.16)
project(entropy-fcitx5 LANGUAGES CXX)

find_package(Fcitx5Core REQUIRED)

add_fcitx5_addon(entropyuniversalsymbols src/entropyuniversalsymbols.cpp)
target_link_libraries(entropyuniversalsymbols Fcitx5::Core)
install(TARGETS entropyuniversalsymbols DESTINATION "${FCITX_INSTALL_ADDONDIR}")
install(FILES entropyuniversalsymbols.conf DESTINATION "${FCITX_INSTALL_PKGDATADIR}/addon")
