export function Channel(audio_uri) {
    this.audio_uri = audio_uri;
    this.resource = new Audio(audio_uri);
    this.resource.volume = 0.4;
}

Channel.prototype.play = function() {
    this.resource.play();
}

export function MultiAudio(audio_uri, num) {
    this.channels = [];
    this.num = num;
    this.index = 0;

    for (var i = 0; i < num; i++) {
        this.channels.push(new Channel(audio_uri));
    }
}

MultiAudio.prototype.play = function() {
    this.channels[this.index++].play();
    this.index = this.index < this.num ? this.index : 0;
}