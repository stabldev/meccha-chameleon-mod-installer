$DownloadUrl = "https://files.catbox.moe/8vd2ov.zip"
$ModpackName = "modpack.zip"
$TempZipPath = Join-Path -Path ([System.IO.Path]::GetTempPath()) -ChildPath $ModpackName

function Get-GameFolder {
  Add-Type -AssemblyName System.Windows.Forms
  $dialog = New-Object System.Windows.Forms.FolderBrowserDialog
  $dialog.Description = "Select Meccha Chameleon installation folder"
  $dialog.ShowNewFolderButton = $true
  if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) { return $dialog.SelectedPath }
  return $null
}

function Save-Modpack {
  try {
    Write-Host "Downloading modpack from $DownloadUrl..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempZipPath -ErrorAction Stop
    Write-Host "Download successful." -ForegroundColor Green
    return $true
  }
  catch {
    Write-Host "Error downloading modpack: $($_.Exception.Message)" -ForegroundColor Red
    return $false
  }
}

function Expand-Modpack {
  param ([string]$Destination)
  try {
    Write-Host "Extracting to $Destination..." -ForegroundColor Cyan
    Add-Type -AssemblyName System.IO.Compression.FileSystem

    $zip = [System.IO.Compression.ZipFile]::OpenRead($TempZipPath)
    $entries = $zip.Entries
    $totalCount = $entries.Count
    $currentCount = 0
    $overwriteAll = $false
    $skipAll = $false

    foreach ($entry in $entries) {
      $currentCount++
      $percent = [math]::Round(($currentCount / $totalCount) * 100)
      Write-Progress -Activity "Extracting Modpack" -Status "$percent% - $($entry.FullName)" -PercentComplete $percent

      $destPath = [System.IO.Path]::GetFullPath([System.IO.Path]::Combine($Destination, $entry.FullName))
      if (-not $destPath.StartsWith([System.IO.Path]::GetFullPath($Destination))) { continue }

      if ($entry.FullName.EndsWith("/") -or $entry.FullName.EndsWith("\")) {
        if (-not (Test-Path -Path $destPath)) { New-Item -ItemType Directory -Path $destPath -Force | Out-Null }
        continue
      }

      $parentDir = Split-Path -Path $destPath -Parent
      if (-not (Test-Path -Path $parentDir)) { New-Item -ItemType Directory -Path $parentDir -Force | Out-Null }

      $doExtract = $true
      if (Test-Path -Path $destPath) {
        if ($overwriteAll) { $doExtract = $true }
        elseif ($skipAll) { $doExtract = $false }
        else {
          Write-Progress -Activity "Extracting Modpack" -Completed
          $choices = [System.Management.Automation.Host.ChoiceDescription[]] @("&Yes", "&No", "Yes to &All", "N&o to All")
          $choice = $Host.UI.PromptForChoice("Confirm Overwrite", "File exists:`n$destPath`n`nOverwrite?", $choices, 0)
          switch ($choice) {
            0 { $doExtract = $true }
            1 { $doExtract = $false }
            2 { $doExtract = $true; $overwriteAll = $true }
            3 { $doExtract = $false; $skipAll = $true }
          }
        }
      }

      if ($doExtract) {
        [System.IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $destPath, $true)
      }
    }
    
    $zip.Dispose()
    Write-Progress -Activity "Extracting Modpack" -Completed
    Write-Host "Extraction completed successfully." -ForegroundColor Green
    return $true
  }
  catch {
    Write-Progress -Activity "Extracting Modpack" -Completed
    Write-Host "Error during extraction: $($_.Exception.Message)" -ForegroundColor Red
    if ($null -ne $zip) { $zip.Dispose() }
    return $false
  }
}

Clear-Host
Write-Host "=== Meccha Chameleon Mod Installer ===" -ForegroundColor Yellow
Write-Host "---------------------- by stabldev ---" -ForegroundColor DarkGray

$targetFolder = Get-GameFolder
if ([string]::IsNullOrWhiteSpace($targetFolder)) {
  Write-Host "Installation canceled. No folder selected." -ForegroundColor Yellow
  Write-Host "Press any key to exit..." -ForegroundColor Gray
  $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
  exit
}

Write-Host "Selected Folder: $targetFolder" -ForegroundColor Cyan

if (Save-Modpack) {
  if (Expand-Modpack -Destination $targetFolder) {
    Write-Host "Modpack Installation Finished Successfully!" -ForegroundColor Green
  }
}

if (Test-Path -Path $TempZipPath) {
  Write-Host "Cleaning up temporary files..." -ForegroundColor Gray
  Remove-Item -Path $TempZipPath -Force -ErrorAction SilentlyContinue
}

Write-Host "Press any key to exit..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
