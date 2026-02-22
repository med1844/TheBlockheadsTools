# Fix Rotation

If you also uses the 1.7.6 APK from [APKPure](https://apkpure.com/the-blockheads/com.noodlecake.blockheads/versions), you might find the game never goes to the landscape mode in bluestacks 4.

Thankfully this is solvable by patching the APK.

!!! warning
    This will require you uninstall the game first as the signature will be different. With [root access](./setup-emulator.md), you should be able to backup everything before uninstalling the game.

## Prerequisites

Ensure you have java runtime installed, the APK available on your PC, then download [apktool](https://apktool.org/) and [uber-apk-signer](https://github.com/patrickfav/uber-apk-signer).

## Decode APK

In the folder you stored the apktool, run:

```powershell
java -jar .\apktool_2.12.1.jar d 'C:\path\to\your\apk\The Blockheads_1.7.6_APKPure.apk'
```

This should generate a folder named `The Blockheads_1.7.6_APKPure` in `C:\path\to\your\apk\`:

```
> tree 'The Blockheads_1.7.6_APKPure' -L 1
The Blockheads_1.7.6_APKPure/
├── AndroidManifest.xml
├── apktool.yml
├── assets
├── build
├── lib
├── original
├── res
├── smali
├── smali_assets
├── smali_classes2
├── smali_classes3
└── unknown
```

## Edit manifest

Open `AndroidManifest.xml` with your favorite text editor, edit these lines:

```diff
- <meta-data android:name="apportable.orientation" android:value="portrait"/>
+ <meta-data android:name="apportable.orientation" android:value="any"/>
  <meta-data android:name="apportable.opengles2" android:value="true"/>
  <!-- ... -->
  <meta-data android:name="com.google.android.gms.games.APP_ID" android:value="@string/app_id"/>
  <activity
      android:label="@string/app_name"
      android:launchMode="singleTask"
      android:name="com.apportable.activity.VerdeActivity"
-     android:screenOrientation="portrait"
+     android:screenOrientation="sensorLandscape"
      android:windowSoftInputMode="adjustNothing" android:configChanges="mcc|mnc|locale|touchscreen|keyboard|keyboardHidden|navigation|orientation|screenLayout|uiMode|screenSize|smallestScreenSize|fontScale">
```

## Rebuild & sign APK

Rebuild the APK by running:

```powershell
java -jar .\apktool_2.12.1.jar b 'C:\path\to\your\apk\The Blockheads_1.7.6_APKPure' -o blockheads_modified.apk
```

Then, sign the new APK with uber-apk-signer:

```powershell
java -jar .\uber-apk-signer-1.3.0.jar --apks .\blockheads_modified.apk
```

You should get a file named `blockheads_modified-aligned-debugSigned.apk`, install that in bluestack4, you should now be able to have the game run in landscape mode.
