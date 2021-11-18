fn main() {
	windows::build! {
        Windows::Win32::UI::WindowsAndMessaging::EnumWindows,
        Windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId,
        Windows::Win32::UI::WindowsAndMessaging::IsWindowVisible,
        Windows::Win32::UI::WindowsAndMessaging::SetParent,
    };
}