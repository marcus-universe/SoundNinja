// import {Howl, Howler} from 'howler';
const {ipcRenderer} = require('electron');
const dialog = require('electron').remote.dialog
const fs = require('fs');
const path = require('path');
const shell = require('electron').shell;

// const Tone = require('tone');
var {Howl} = require('howler');


// const HorizontalScroll = require('horizontal-scroll');


const SoundButtonContainer = document.querySelector('#SoundButton_Container');
var TabsList = document.querySelectorAll(".Tab")
const CreateDiv = document.createElement("DIV");
const CreateButton = document.createElement("DIV");

//Color Picker
let root = document.documentElement;
const Colorpicker = document.querySelector('#Colorpicker')
// var ColorpickerValue
let currentName = ""
var SettingData
var FolderList

var foldername = []
var folderpath = []
var audioData
// var audioPath

var folderpathString = "";
var colorTheme

// var ButtonNumber = 0;

//?##########################
//? ANCHOR Web Audio
//?##########################
// const AudioContext = window.AudioContext || window.webkitAudioContext;
// const ctx = new AudioContext();










//?##########################
//? ANCHOR Read in Json File
//?##########################
fs.readFile('config.json', 'utf-8', function (err, data) {
    if (err) throw err
    SettingData = JSON.parse(data);
    FolderList = SettingData.folder
    colorTheme = SettingData.settings[0].color
    setColorTheme(colorTheme);
    console.log(FolderList)
    for (let i = 0; i < FolderList.length; i++) {
        currentName = FolderList[i].name;
        // console.log(currentName);
        for (let c = 0; c < CreateDiv.classList.length; c++) {
            CreateDiv.removeAttribute("class");
        }
        CreateDiv.classList.add("Tab");
        CreateDiv.classList.add("flex_c_h");
        CreateDiv.setAttribute('id', currentName);
        TabsContainer.appendChild(CreateDiv.cloneNode(true)).innerHTML = currentName;
        // console.log(currentName)
        // var AudioTest = fs.readdirSync(FolderList[i].audiofiles)
        // console.log("Create Tab")
        audioData = FolderList[i].audiofiles
        CreateButtons(audioData);
        // console.log(audioData)
    }




    TabsList = document.querySelectorAll(".Tab")
})


//?##########################
//? ANCHOR Write in Json File
//?##########################
function WriteJson() {
    fs.writeFile('config.json', JSON.stringify(SettingData, null, 2), 'utf-8', function (err) {
        if (err) throw err
        console.log('Done!')
    })
}

//?##########################
//?ANCHOR Select Folder
//?##########################
function ReadFiles(folderpath, a) {
                let audioData = fs.readdirSync(folderpath[a])
                filestoJSON(audioData)

            }

var JsonStruct = {
                name: foldername,
                foldername: foldername,
                path: folderpathString,
                audiofiles: []
            }


function filestoJSON(audioData) {

                for (let i = 0; i < audioData.length; i++) {
                    let audioPath = folderpathString + "\\" + audioData[i]
                    JsonStruct.audiofiles.push({
                        audioname: audioData[i],
                        audiourl: audioPath
                    });
                }

            }

document.querySelector('#Folder').addEventListener('click', () => {

    //?Open Dialog Explorer
    dialog.showOpenDialog({
        properties: ['openDirectory', 'multiSelections']
    }).then(result => { //?Get Selected Folders

        folderpath = result.filePaths

        for (let a = 0; a < folderpath.length; a++) {
            foldername = path.basename(folderpath[a].toString());
            folderpathString = folderpath[a].toString();

            CreateDiv.classList.add("Tab");
            CreateDiv.classList.add("flex_c_h");
            CreateDiv.setAttribute("id", foldername.toString());
            TabsContainer.appendChild(CreateDiv.cloneNode(true)).innerHTML = foldername;



            


            
            ReadFiles(folderpath, a);

            
            SettingData.folder.push(JsonStruct)

            WriteJson();

            // var AudioListJson = SettingData.folder[a].audiofiles
            // CreateButtons(AudioListJson);

        }
        location.reload();
        return false;
    })
})

//?##########################
//? ANCHOR Scroll Tabs
//?##########################
TabsContainer.addEventListener("wheel", (evt) => {
    evt.preventDefault();
    TabsContainer.scrollLeft += evt.deltaY;
});

//?##########################
//? ANCHOR Reset Json File
//?##########################
document.querySelector('#Reset').addEventListener('click', () => {
    fs.readFile('config.json', 'utf-8', function (err, data) {
        if (err) throw err
        SettingData = JSON.parse(data);

    })


    for (let i = 0; i < SettingData.folder.length; i++) {
        console.log(i)
        SettingData.folder.splice(i, 99999999)

    }
    SettingData.settings[0].color = "#2adfff"
    WriteJson()
    location.reload();
    return false;
});





//?##########################
//? ANCHOR Buttons
//?##########################

// ?Add Button


function CreateButtons(audioData) {
    // var ButtonNumber = 0;
    for (let b = 0; b < audioData.length; b++) {
        for (let c = 0; c < CreateButton.classList.length; c++) {
            CreateButton.removeAttribute("class");
        }
        CreateButton.classList.add("SoundButton");
        CreateButton.classList.add("grid_c");
        CreateButton.setAttribute("id", currentName);
        CreateButton.setAttribute("data-type", audioData[b].audiourl);



        SoundButtonContainer.appendChild(CreateButton.cloneNode(true)).innerHTML = audioData[b].audioname.replaceAll('.wav', '').replaceAll('.mp3', '').replaceAll("_", " ");
    }

}


// NodeList.prototype.forEach = Array.prototype.forEach
// var children = SoundButtonContainer.childElementCount;



// ?Clicked Button

SoundButtonContainer.addEventListener('click', SwitchButton, false);

// var player
var audiolink
var soundList = {}


function SwitchButton(e) {
    // var playing = false;



    if (e.target !== e.currentTarget) {
        var clickedButton = e.target;
        console.log(clickedButton)




        if (clickedButton.classList.contains("SoundButtonActive")) {
            clickedButton.classList.remove("SoundButtonActive");
            audiolink = clickedButton.getAttribute("data-type").toString();
            soundList.stop();
        } else {
            clickedButton.classList.add("SoundButtonActive");

            audiolink = clickedButton.getAttribute("data-type").toString();

            playcurrentSound(audiolink, clickedButton)
            // console.log(soundList)
            // console.log(audiolink)

            // player = new Tone.Player(audiolink).toDestination()

            // Tone.loaded().then(() => {
            //     player.start(0); 
            // });


        }

    }
    e.stopPropagation();
}


function playcurrentSound(audiolink, clickedButton) {

    soundList = new Howl({
        src: [audiolink],
        html5: true,
        preload: true,
        onend: function () {
            clickedButton.classList.remove("SoundButtonActive");
        }
    });

    soundList.play();
    console.log(soundList)
}


//?##########################
//? ANCHOR StopAll Sounds
//?##########################
const StopButton = document.querySelectorAll('#Stop')

StopButton.forEach(stopbtn => {
    stopbtn.addEventListener('click', () => {
        
        for (var i = soundList.length - 1; i >= 0; i--) {
            Object.keys(soundList).forEach(function (key) {
                soundList[key].stop();
                console.log([key])
            });
        }
        console.log("Test")
    })
})





//?##########################
//? ANCHOR Tabs
//?##########################
// const AllButtons = document.querySelectorAll('.SoundButton');

TabsContainer.addEventListener('click', TabsSelecting, false);

function TabsSelecting(e) {
    if (e.target !== e.currentTarget) {
        var clickedTab = e.target;
        var idTab = clickedTab.getAttribute("id")

        if (clickedTab.classList.contains("SoundButtonActive")) {
            clickedTab.classList.remove("SoundButtonActive");

            SoundButtonContainer.childNodes.forEach(ChildButton => {
                ChildButton.style.display = "grid"
            })

        } else {
            for (var i = 0; i < TabsList.length; i++) {
                TabsList[i].classList.remove("SoundButtonActive");
            }
            clickedTab.classList.add("SoundButtonActive");

            SoundButtonContainer.childNodes.forEach(ChildButton => {
                if (ChildButton.id.indexOf(idTab)) {
                    ChildButton.style.display = "none"
                } else {
                    ChildButton.style.display = "grid"
                }
            })
        }
    }
    e.stopPropagation();
}



//?##########################
//? ANCHOR Settings
//?##########################

//?Settings Button
const Settingsbutton = document.querySelector('#Settings');
const SettingPanel = document.querySelector('.SettingPanel');
const SettingBox = document.querySelector('.SettingBox');
const ExitSettings = document.querySelector('.ExitSetting');

Settingsbutton.addEventListener('click', () => {
    SettingPanel.classList.add("ActiveSettings");
    SettingBox.classList.add("ActiveSettings")
    console.log("SettingsPage")
})

ExitSettings.addEventListener('click', () => {
    SettingPanel.classList.remove("ActiveSettings");
    SettingBox.classList.remove("ActiveSettings")
})




//?Color Picker
function setColorTheme(colorTheme) {
    root.style.setProperty('--primary_color', colorTheme)
    Colorpicker.value = colorTheme
}

Colorpicker.addEventListener('input', () => {
    // var ColorpickerValue = Colorpicker.value;
    root.style.setProperty('--primary_color', Colorpicker.value)
    fs.readFile('config.json', 'utf-8', function (err, data) {
        if (err) throw err
        SettingData = JSON.parse(data);
        colorTheme = SettingData.settings[0].color
        SettingData.settings[0].color = Colorpicker.value
        WriteJson();
    })
})




//?Select Audiodevice
const AudioSelector = document.querySelector('select#audioSource')
const selectors = [AudioSelector]

function gotDevices(deviceInfos) {
    // Handles being called several times to update labels. Preserve values.
    // const values = selectors.map(select => select.value);
    selectors.forEach(select => {
        while (select.firstChild) {
            select.removeChild(select.firstChild);
        }
    });

    for (let i = 0; i !== deviceInfos.length; ++i) {

        const deviceInfo = deviceInfos[i];
        const option = document.createElement('option');
        option.value = deviceInfo.deviceId;

        if (deviceInfo.kind === 'audiooutput') {
            option.text = deviceInfo.label || `speaker ${AudioSelector.length + 1}`;
            AudioSelector.appendChild(option);
        }
    }

    return navigator.mediaDevices.enumerateDevices();
}

navigator.mediaDevices.enumerateDevices().then(gotDevices)


// function handleError(error) {
//     console.log('navigator.MediaDevices.getUserMedia error: ', error.message, error.name);
//   }



  function attachSinkId(element, sinkId) {
    if (typeof element.sinkId !== 'undefined') {
      element.setSinkId(sinkId)
          .then(() => {
            console.log(`Success, audio output device attached: ${sinkId} to element with ${element.title} as source.`);
          })
          .catch(error => {
            let errorMessage = error;
            if (error.name === 'SecurityError') {
              errorMessage = `You need to use HTTPS for selecting audio output device: ${error}`;
            }
            console.error(errorMessage);
            // Jump back to first output device in the list as it's the default.
            // outputSelector.selectedIndex = 0;
          });
    } else {
      console.warn('Browser does not support output device selection.');
    }
  }



  function changeaudio(){
        const DeviceTarget = AudioSelector.target
        SettingData.settings[0].inputaudio = AudioSelector.value
        WriteJson();
        const audioDestination = AudioSelector.value;
        const AudioEvent = AudioSelector.childNodes[1]
        attachSinkId( AudioEvent, audioDestination, DeviceTarget);
  }

  

  AudioSelector.onchange = changeaudio





//?##########################
//? ANCHOR About
//?##########################
const AboutButton = document.querySelector('.About');
const AboutContainer = document.querySelector('.AboutContainer');
const AboutPage = document.querySelector('.AboutPage');
const ExitAbout = document.querySelector('.ExitAbout');

AboutButton.addEventListener('click', () => {
    console.log("AboutPage")
    AboutContainer.classList.add("ActiveSettings");
    AboutPage.classList.add("ActiveSettings")

});

ExitAbout.addEventListener('click', () => {
    AboutContainer.classList.remove("ActiveSettings");
    AboutPage.classList.remove("ActiveSettings");
});

var SocialLinks = document.querySelectorAll('.SupportLinks a')

for (let i = 0; i < SocialLinks.length; i++) {
    SocialLinks[i].addEventListener('click', () => {
        console.log(SocialLinks[i].getAttribute("data"))
        shell.openExternal(SocialLinks[i].getAttribute("data"))

    })
}



//?##########################
//? ANCHOR Maximize
//?##########################
const maximizeButton = document.querySelector('.Maximize')

maximizeButton.addEventListener('click', () => {
    ipcRenderer.send('maximizeapp');
})

//?##########################
//? ANCHOR Hide
//?##########################
const hideButton = document.querySelector('.HideApp');

hideButton.addEventListener('click', () => {
    ipcRenderer.send('hideapp');
})

//?##########################
//? ANCHOR Exit
//?##########################
const ExitButtons = document.querySelectorAll('#Exit')

ExitButtons.forEach(XButton => {
    XButton.addEventListener('click', () => {
        ipcRenderer.send('exitapp');
        console.log("test");
    });
})