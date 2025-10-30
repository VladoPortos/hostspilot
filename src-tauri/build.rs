fn main() {
    // For Windows, embed manifest requesting admin rights
    #[cfg(windows)]
    {
        let mut windows = tauri_build::WindowsAttributes::new();
        windows = windows.app_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0" xmlns:asmv3="urn:schemas-microsoft-com:asm.v3">
  <assemblyIdentity version="1.0.0.0" name="HostsPilot.app" type="win32" />
  <description>HostsPilot - Manage your hosts files</description>
  <asmv3:trustInfo>
    <asmv3:security>
      <asmv3:requestedPrivileges>
        <asmv3:requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
      </asmv3:requestedPrivileges>
    </asmv3:security>
  </asmv3:trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
      <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}"/>
      <supportedOS Id="{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}"/>
      <supportedOS Id="{35138b9a-5d96-4fbd-8e2d-a2440225f93a}"/>
    </application>
  </compatibility>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*" />
    </dependentAssembly>
  </dependency>
</assembly>
"#);
        tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
            .expect("failed to run tauri-build");
    }
    
    #[cfg(not(windows))]
    {
        tauri_build::build()
    }
}
