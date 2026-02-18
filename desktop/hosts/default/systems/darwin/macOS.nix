{
  system.defaults = {
    CustomSystemPreferences = {
      NSGlobalDomain = {
        ApplePressAndHoldEnabled = false;
        AppleScrollerPagingBehavior = true;
      };
    };

    CustomUserPreferences = {
      # XXX: Prevent automatic Dock restart by nix-darwin
      #      Run dock manually if config changed: killall Dock
      "com.apple.dock" = {
        appswitcher-all-displays = false;
        autohide = true;
        autohide-delay = 0.0;
        expose-group-apps = true;
        magnification = false;
        mineffect = "scale";
        minimize-to-application = false;
        mouse-over-hilite-stack = true;
        mru-spaces = false;
        orientation = "bottom";
        show-recents = true;
        static-only = false;
        tilesize = 64;

        # hot corners
        wvous-tl-corner = 1; # Top Left: Disabled
        wvous-tr-corner = 4; # Top Right: Desktop
        wvous-bl-corner = 2; # Bottom Left; Mission Control
        wvous-br-corner = 3; # Bottom Right: Application Windows
      };
    };

    ".GlobalPreferences" = {
      "com.apple.mouse.scaling" = 1.0;
      "com.apple.sound.beep.sound" = "/System/Library/Sounds/Tink.aiff";
    };

    NSGlobalDomain = {
      AppleEnableMouseSwipeNavigateWithScrolls = true;
      AppleICUForce24HourTime = true;
      AppleInterfaceStyleSwitchesAutomatically = true;
      AppleKeyboardUIMode = 3;
      AppleShowAllExtensions = true;
      AppleShowScrollBars = "WhenScrolling";
      AppleSpacesSwitchOnActivate = true;
      AppleWindowTabbingMode = "manual";
      InitialKeyRepeat = 15;
      KeyRepeat = 2;
      NSAutomaticCapitalizationEnabled = false;
      NSAutomaticDashSubstitutionEnabled = false;
      NSAutomaticInlinePredictionEnabled = false;
      NSAutomaticPeriodSubstitutionEnabled = false;
      NSAutomaticQuoteSubstitutionEnabled = false;
      NSAutomaticSpellingCorrectionEnabled = false;
      NSAutomaticWindowAnimationsEnabled = true;
      NSScrollAnimationEnabled = true;
      NSWindowShouldDragOnGesture = true;
      "com.apple.keyboard.fnState" = true;
      "com.apple.sound.beep.feedback" = 1;
      "com.apple.springing.delay" = 0.5;
      "com.apple.springing.enabled" = true;
      "com.apple.swipescrolldirection" = true;
    };

    WindowManager = {
      AutoHide = false;
      EnableStandardClickToShowDesktop = true;
      EnableTiledWindowMargins = true;
      GloballyEnabled = false;
      HideDesktop = false;
      StageManagerHideWidgets = false;
      StandardHideDesktopIcons = false;
      StandardHideWidgets = false;
    };

    SoftwareUpdate = {
      AutomaticallyInstallMacOSUpdates = false;
    };

    menuExtraClock = {
      IsAnalog = false;
      Show24Hour = true;
      ShowAMPM = false;
      ShowDayOfMonth = false;
      ShowDayOfWeek = true;
      ShowDate = 1; # Always
      ShowSeconds = false;
    };

    controlcenter = {
      AirDrop = false;
      BatteryShowPercentage = false;
      Bluetooth = false;
      Display = false;
      FocusModes = true;
      NowPlaying = true;
      Sound = false;
    };

    # XXX: Prevent automatic Dock restart by nix-darwin
    #      See `system.defaults.CustomSystemPreferences."com.apple.dock"`
    # dock = { ... };

    finder = {
      _FXShowPosixPathInTitle = true;
      _FXSortFoldersFirst = true;
      _FXSortFoldersFirstOnDesktop = true;
      AppleShowAllExtensions = true;
      CreateDesktop = true; # Show icons on desktop
      FXDefaultSearchScope = "SCcf"; # Search current folder
      FXEnableExtensionChangeWarning = true;
      FXPreferredViewStyle = "Nlsv"; # List view
      NewWindowTarget = "Home";
      ShowExternalHardDrivesOnDesktop = false;
      ShowHardDrivesOnDesktop = false;
      ShowMountedServersOnDesktop = false;
      ShowPathbar = true;
      ShowRemovableMediaOnDesktop = true;
      ShowStatusBar = true;
    };

    hitoolbox = {
      AppleFnUsageType = "Do Nothing";
    };

    loginwindow = {
      DisableConsoleAccess = false;
      GuestEnabled = false;
      PowerOffDisabledWhileLoggedIn = false;
      RestartDisabled = false;
      RestartDisabledWhileLoggedIn = false;
      ShutDownDisabled = false;
      ShutDownDisabledWhileLoggedIn = false;
      SleepDisabled = false;
    };

    magicmouse = {
      MouseButtonMode = "TwoButton";
    };

    screencapture = {
      disable-shadow = true;
      include-date = true;
      location = null;
      show-thumbnail = true;
      target = "clipboard";
      type = "png";
    };

    screensaver = {
      askForPassword = true;
      askForPasswordDelay = 5;
    };

    spaces = {
      spans-displays = false;
    };

    trackpad = {
      ActuationStrength = 1; # disable
      Clicking = true;
      Dragging = false;
      FirstClickThreshold = 1; # medium
      SecondClickThreshold = 1; # medium
      TrackpadRightClick = true;
      TrackpadThreeFingerDrag = false;
    };
  };
}
